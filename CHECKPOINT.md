# GroupMe Pirates Bot Update - Checkpoint
Date: 2025-10-21
Location: /home/rent/rentserv_deployment/services/groupme-bot/groupme-pirates-bot

## OBJECTIVES
1. Add conversational response capability (TARS/CASE-style humor)
2. Move all data to Google Sheets (make calendar optional)
3. Change data priority to sheets-first (calendar augments if provided)
4. Allow manual sheet updates without correlation conflicts

## PROGRESS COMPLETED
1. ✅ Added ConversationalResponse variant to ParsedIntent enum in src/conversational_parser.rs
   - Line 18: Added 'ConversationalResponse { message: String },'

2. ✅ Created backup: src/conversational_parser.rs.pre-update

## FILES MODIFIED
- src/conversational_parser.rs (enum updated)

## FILES WITH UNCOMMITTED CHANGES (per git status)
- src/conversational_parser.rs
- src/error.rs
- src/main.rs
- src/parser.rs

## BACKUP FILES CREATED
- src/conversational_parser.rs.backup
- src/conversational_parser.rs.bak
- src/conversational_parser.rs.pre-update
- src/parser.rs.bak

## NEXT STEPS

### 1. Complete Conversational Response Feature
File: src/conversational_parser.rs
- Insert methods before line 382 (before 'fn is_help_intent'):
  * is_conversational_message() - detect conversational keywords
  * get_conversational_response() - router for response types
  * get_fear_response() - responds to scared/afraid
  * get_humor_response() - responds to jokes/funny
  * get_thanks_response() - responds to thanks
  * get_positive_response() - responds to love/awesome
  * get_negative_response() - responds to hate/terrible
  * get_generic_conversational_response() - default responses

- Update detect_intent() method (around line 69):
  Add before 'ParsedIntent::Unknown':
  if self.is_conversational_message(text_lower) {
      let response = self.get_conversational_response(text_lower);
      return ParsedIntent::ConversationalResponse { message: response };
  }

File: src/parser.rs
- Add match arm in intent_to_command() (after line 58):
  ParsedIntent::ConversationalResponse { message } => {
      Err(BotError::InvalidCommand(message))
  }

### 2. Make Calendar Optional
File: src/config.rs
- Change calendar_webcal_url from String to Option<String>
- Update from_env() to use env::var().ok() instead of requiring it

### 3. Update Data Fetching - Sheets First
File: src/google_client.rs
- Modify get_calendar_events() to return empty vec if calendar_url is None
- Add function: import_calendar_to_sheet() for bulk loading

File: src/service.rs
- Change correlate_data() logic:
  * Start with sheets data as base
  * Add calendar events that aren't in sheets (if calendar provided)
  * Preserve all sheet data (allow manual edits)

## IMPLEMENTATION NOTES

### Conversational Response Style
- Based on TARS/CASE from Interstellar
- Humor setting at 95 percent
- Mixes pirate theme with sarcastic AI personality
- Examples:
  * 'Scared of a pirate bot? That is adorable. My humor setting is at 90 percent.'
  * 'TARS: Everybody good? Me: Define good. Does forgetting snacks count?'
  * iPhone autocorrect jokes (inappropriate/witty)

### Current Data Flow Issue
- Calendar is baseline, sheets augment
- This prevents postseason manual updates
- Need to flip: sheets baseline, calendar augments

### Testing Commands
After implementation:
- cargo build
- cargo test
- Test conversational: '@PirateBot I am scared of you'
- Test manual sheet updates without correlation blocking

## TECHNICAL CONTEXT
- Language: Rust
- Framework: Actix-web
- APIs: Google Sheets API, Google Calendar (iCal)
- Message flow: GroupMe webhook -> parse_message -> handle_command -> send_response

## ISSUES ENCOUNTERED
- Heredoc commands with large code blocks failed to execute
- Interactive cat/editor commands timed out
- Solution: Break into smaller edits or manual editing required

## RECOMMENDATIONS FOR CONTINUATION
1. Review git diff to see what's already changed
2. Use nano/vim for manual edits (editors confirmed available)
3. Or create separate .patch files and apply with 'git apply'
4. Test incrementally - build after each major change

## CONVERSATION CONTEXT
- User: 'rent' on Ubuntu Linux
- Bot: Rust-based GroupMe bot for Pirates baseball team
- Users are engaging bot, need it to respond to casual messages
- Postseason started, calendar not updating, need sheets-only option
- User wants TARS/CASE personality (witty, almost inappropriate)

