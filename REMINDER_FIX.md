# Reminder Team Name Fix - October 7, 2025

## Issue
Reviewing the bot logs, the 24-hour reminder that was sent showed:
```
⏰ Game Reminder! 24 hours until:

🏴‍☠️ 6pm-7:30pm at MZ1 1307 Greenville Street McKinney, TX 75069
...
Home Team: home
```

**No team names were displayed in the title**, just the time and location. The reminder should have shown something like "Pirates (Home) vs Chaos 8U" instead.

## Root Cause Analysis

### The Data Flow Problem
1. **Calendar events** come from TeamSideline with full team info:
   - Format: `" Vs Chaos 8U - Hall (Pirates - Hines)"`
   - Contains: opponent name, field, home team, coach

2. **Google Sheets data** has detailed volunteer info but limited team info:
   - Contains: time, location, **"home"** (as placeholder), volunteers

3. **The Bug**: In `correlate_data()` function (service.rs lines 82-84):
   ```rust
   // Update the event summary to be more descriptive using sheet data
   if !time.is_empty() && !home_team.is_empty() {
       event.event_summary = format!("{} - {}", time, home_team);
   }
   ```
   
   This code was **overwriting** the calendar summary (`" Vs Chaos 8U - Hall (Pirates - Hines)"`) with sheet data (`"6pm-7:30pm - home"`).

4. **Result**: When `format_matchup()` tried to parse team names, it received:
   - ❌ `"6pm-7:30pm - home"` (unparseable)
   - Instead of ✅ `" Vs Chaos 8U - Hall (Pirates - Hines)"` (parseable)

## Solution
Commented out the code that overwrites `event_summary` with sheet data:
```rust
// PRESERVE CALENDAR SUMMARY:                 
// Update the event summary to be more descriptive using sheet data
// PRESERVE CALENDAR SUMMARY:  if !time.is_empty() && !home_team.is_empty() {
// PRESERVE CALENDAR SUMMARY:      event.event_summary = format!("{} - {}", time, home_team);
// PRESERVE CALENDAR SUMMARY:  }
```

Now the calendar summary (with team names) is preserved, allowing `format_matchup()` to correctly parse:
- Home team: "Pirates"  
- Away team: "Chaos 8U"

## Expected Results
The next 24-hour reminder should display:
```
⏰ Game Reminder! 24 hours until:

🏴‍☠️ Pirates (Home) vs Chaos 8U
Date: 2025-10-08
Time: 6pm-7:30pm
Location: MZ1 1307 Greenville Street McKinney, TX 75069
...
```

## Files Changed
- `src/service.rs` - Commented out lines 82-84 that overwrite event_summary

## Related Commits
- `78af065` - Preserve calendar summary to enable team name parsing in reminders
- `0fd425e` - Fix team name parsing to handle TeamSideline calendar format

## Testing
- ✅ Code compiles successfully
- ✅ Docker container rebuilt and deployed
- ✅ Bot running healthy
- ⏳ Next reminder will validate the fix (should show team names in title)

## Why Both Fixes Were Needed
1. **First fix (models.rs)**: Updated `parse_matchup()` to understand TeamSideline calendar format
2. **Second fix (service.rs)**: Ensured calendar summary is preserved so `parse_matchup()` can actually parse it

Without BOTH fixes, team names wouldn't display properly in reminders.
