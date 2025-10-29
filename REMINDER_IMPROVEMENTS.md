# Reminder Service Improvements

## Overview
This document summarizes the improvements made to the reminder service to handle edge cases, use dynamic team configuration, and fetch fresh data.

## Changes Made

### 1. TBD Time Handling
**Problem**: Reminders would default to noon for unparsable times, causing incorrect notifications.

**Solution**:
- Added explicit checks for "TBD" or empty time values
- Gracefully skip reminders when time cannot be determined
- Return errors instead of defaulting to noon in `parse_game_datetime()`
- Log warnings for unparsable times

**Code Location**: `src/reminder.rs` lines 107-119

### 2. Dynamic Team Configuration
**Problem**: Hardcoded Pirates emoji (🏴‍☠️) and team references in reminder messages.

**Solution**:
- Integrated `TeamFactsProvider` into `ReminderScheduler`
- Replaced hardcoded emoji with `self.config.team_emoji`
- Replaced "Pirates" with `self.config.team_name`
- 15-minute reminders now include dynamic team facts when enabled

**Code Location**: `src/reminder.rs`
- Lines 22, 31-40 (TeamFactsProvider integration)
- Lines 172 (24-hour reminder)
- Lines 181-193 (15-minute reminder with team facts)

### 3. Fresh Data Fetching
**Problem**: Reminders used cached data, missing recent schedule updates.

**Solution**:
- Changed from `find_next_event()` to `correlate_data()` 
- Always fetch fresh data from Google Calendar and Sheets
- Manually find next event from fresh data
- Ensures reminders reflect latest information even if external data changes

**Code Location**: `src/reminder.rs` lines 87-104

### 4. Better Error Handling
**Improvements**:
- Skip reminders for games with TBD times instead of failing
- Log informative messages for debugging
- Handle date parsing errors gracefully
- Prevent reminder service from crashing on invalid data

## Testing

All changes have been:
- ✅ Compiled successfully with Rust release profile
- ✅ Tested with existing unit tests (all passing)
- ✅ Code warnings minimized (only unused helper functions remain)

## Configuration

The reminder service now respects these `.env` settings:
- `TEAM_NAME` - Team name used in reminder messages
- `TEAM_EMOJI` - Emoji used in reminder messages  
- `ENABLE_TEAM_FACTS` - Whether to include team facts in 15-minute reminders
- `TEAM_FACTS_FILE` - Optional custom facts JSON file

## Example Output

**24-Hour Reminder**:
```
⏰ Game Reminder! 24 hours until:

🏴‍☠️ Pirates vs Cardinals
Time: 7:30 PM
Location: PNC Park (https://maps.google.com/?q=PNC+Park)
Home Team: Home

⚠️ Still needed: snacks, scoreboard
```

**15-Minute Reminder** (with team facts enabled):
```
⚾ Game starting in 15 minutes! 🏴‍☠️

🏴‍☠️ The Pittsburgh Pirates were founded in 1881, making them one of the oldest franchises in Major League Baseball!

⚾ Let's go Pirates! 🏴‍☠️
```

## Future Enhancements

Potential improvements for consideration:
1. Add manual message cleanup command to keep chat tidy
2. Support timezone configuration for multi-region teams
3. Add configurable reminder intervals (not just 24h and 15m)
4. Support multiple notification channels
