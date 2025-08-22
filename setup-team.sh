#!/bin/bash

# GroupMe Bot Team Setup Script
# Usage: ./setup-team.sh <team-name> <base-domain>
# Example: ./setup-team.sh pirates rentbox.us

set -e

if [ $# -ne 2 ]; then
    echo "Usage: $0 <team-name> <base-domain>"
    echo ""
    echo "Examples:"
    echo "  $0 pirates rentbox.us     -> piratesbot.rentbox.us"
    echo "  $0 dragons myteam.com     -> dragonsbot.myteam.com"
    echo "  $0 eagles sportsbot.net   -> eaglesbot.sportsbot.net"
    exit 1
fi

TEAM_NAME="${1,,}"  # Convert to lowercase
BASE_DOMAIN="$2"
BOT_URL="https://${TEAM_NAME}bot.${BASE_DOMAIN}"

echo "🤖 GroupMe Bot Team Setup"
echo "========================="
echo "Team Name: $TEAM_NAME"
echo "Base Domain: $BASE_DOMAIN"
echo "Bot URL: $BOT_URL"
echo ""

# Check if .env exists
if [ ! -f .env ]; then
    if [ -f .env.template ]; then
        echo "📋 Creating .env from template..."
        cp .env.template .env
    else
        echo "❌ No .env.template found. Please create one first."
        exit 1
    fi
fi

echo "⚙️  Updating .env configuration..."

# Update the team-specific variables in .env
if command -v sed >/dev/null 2>&1; then
    # Update or add TEAM_NAME
    if grep -q "^TEAM_NAME=" .env; then
        sed -i "s/^TEAM_NAME=.*/TEAM_NAME=$TEAM_NAME/" .env
    else
        echo "TEAM_NAME=$TEAM_NAME" >> .env
    fi
    
    # Update or add BASE_DOMAIN
    if grep -q "^BASE_DOMAIN=" .env; then
        sed -i "s/^BASE_DOMAIN=.*/BASE_DOMAIN=$BASE_DOMAIN/" .env
    else
        echo "BASE_DOMAIN=$BASE_DOMAIN" >> .env
    fi
    
    # Update GROUPME_BOT_NAME if it's still the template default
    if grep -q "^GROUPME_BOT_NAME=YourBotName" .env; then
        CAPITALIZED_TEAM=$(echo "$TEAM_NAME" | sed 's/.*/\u&/')
        sed -i "s/^GROUPME_BOT_NAME=YourBotName/GROUPME_BOT_NAME=${CAPITALIZED_TEAM}Bot/" .env
        echo "📝 Updated bot name to ${CAPITALIZED_TEAM}Bot (you can change this in .env)"
    fi
    
    echo "✅ Configuration updated!"
else
    echo "❌ sed command not found. Please manually update .env with:"
    echo "   TEAM_NAME=$TEAM_NAME"
    echo "   BASE_DOMAIN=$BASE_DOMAIN"
    exit 1
fi

echo ""
echo "📋 Current configuration:"
echo "========================"
grep -E "^(TEAM_NAME|BASE_DOMAIN|GROUPME_BOT_NAME)=" .env || true
echo ""

echo "⚠️  Next steps:"
echo "1. Edit .env and add your actual API keys and IDs:"
echo "   - GROUPME_BOT_ID"
echo "   - GOOGLE_API_KEY"
echo "   - SHEET_ID"
echo "   - CALENDAR_WEBCAL_URL"
echo ""
echo "2. Make sure DNS is configured:"
echo "   ${TEAM_NAME}bot.${BASE_DOMAIN} -> YOUR_SERVER_IP"
echo ""
echo "3. Deploy the bot:"
echo "   docker-compose up -d                    # Development"
echo "   docker-compose -f docker-compose.prod.yml up -d  # Production"
echo ""
echo "4. Update GroupMe webhook URL:"
echo "   $BOT_URL"
echo ""
echo "🏴‍☠️ Your ${TEAM_NAME} bot will be available at: $BOT_URL"
