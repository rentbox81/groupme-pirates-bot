# Team Name Parsing Fix - October 6, 2025

## Issue
The PirateBot was not correctly parsing team names from the Google Calendar events when responding to "next game" requests. The bot was unable to display proper matchup information showing home vs away teams.

## Root Cause
The calendar events from TeamSideline come in a specific format:
```
" Vs [OpponentTeam] - [Field] ([HomeTeam] - [Coach])"
```

Examples from the actual calendar:
- " Vs Chaos 8U - Hall (Pirates - Hines)"
- " Vs Melissa Cardinals Dirtybirds 8U - Crabb (Pirates - Hines)"

The original `parse_matchup` function was looking for a simple "Team1 vs Team2" pattern, which didn't match the TeamSideline format at all.

## Solution
Updated the `parse_matchup` function in `src/models.rs` to:

1. **Parse the TeamSideline format correctly:**
   - Extract opponent team name (between "Vs " and first " - ")
   - Extract home team name (from inside parentheses, before the second " - ")
   - Return tuple of (HomeTeam, OpponentTeam)

2. **Maintain backward compatibility:**
   - Falls back to the old parsing logic for other calendar formats
   - Ensures the bot continues to work even if calendar format changes

3. **Improved output:**
   - Now displays: "Pirates (Home) vs Chaos 8U"
   - Or: "Pirates (Home) vs Melissa Cardinals Dirtybirds 8U"

## Testing
Created standalone test that verified the parsing logic:
```
Testing:  Vs Chaos 8U - Hall (Pirates - Hines)
  ✓ Parsed successfully!
    Home team: Pirates
    Away team: Chaos 8U

Testing:  Vs Melissa Cardinals Dirtybirds 8U - Crabb (Pirates - Hines)
  ✓ Parsed successfully!
    Home team: Pirates
    Away team: Melissa Cardinals Dirtybirds 8U
```

## Deployment
- Code compiled successfully with no errors
- Docker container rebuilt and redeployed
- Bot is running healthy on port 18080
- Changes committed to git (commit: 0fd425e)

## Next Steps
The next time someone asks the bot for "next game" information in the GroupMe chat, it should now correctly display:
- "Pirates (Home) vs [Opponent Team Name]"

Instead of falling back to generic messages or showing "Home: home" debug strings.

## Files Changed
- `src/models.rs` - Updated `parse_matchup()` function in `CorrelatedEvent` implementation

## Related
This fix addresses the persistent issue with home/away team name display that was noted in the conversation history.
