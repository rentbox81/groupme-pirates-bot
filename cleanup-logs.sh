#!/bin/bash
# Cleanup GroupMe bot logs older than 30 days
find /home/rent/rentserv_deployment/services/groupme-bot/groupme-pirates-bot/logs -name "groupme-bot.log.*" -mtime +30 -delete
