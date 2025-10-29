# GroupMe Pirates Bot - Next Steps & Improvements

## High Priority

- [ ] Add deployment script (deploy.sh) with build, test, restart, rollback
- [x] Fix duplicate parser test issue (main.rs vs lib.rs)
- [x] Update README.md with quick start and env vars
- [x] Document all env vars in .env.template

## Medium Priority

- [ ] Add CI/CD pipeline (GitHub Actions)
- [ ] Add integration tests for full message flow
- [ ] Make volunteer roles configurable (not hardcoded)
- [ ] Add admin commands (refresh data, check status)
- [ ] Fully rewrite correlate_data() to be truly sheets-first
- [ ] Add monitoring and health metrics

## Low Priority

- [ ] Add ARCHITECTURE.md documentation
- [ ] Consider persistent storage/database
- [ ] Service account key rotation
- [ ] Rate limiting on webhooks
- [ ] Interactive help with examples
