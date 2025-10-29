#!/bin/bash

echo "🔍 GroupMe Bot Deployment Verification"
echo "======================================"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print status
print_status() {
    if [ $1 -eq 0 ]; then
        echo -e "✅ ${GREEN}$2${NC}"
    else
        echo -e "❌ ${RED}$2${NC}"
    fi
}

print_warning() {
    echo -e "⚠️  ${YELLOW}$1${NC}"
}

echo "1. Checking required files..."
echo "==============================="

if [ -f ".env" ]; then
    print_status 0 ".env file exists"
else
    print_status 1 ".env file missing - copy from .env.template"
fi

if [ -f "service-account.json" ]; then
    print_status 0 "service-account.json exists"
else
    print_status 1 "service-account.json missing - add Google service account key"
fi

if [ -f "Dockerfile" ]; then
    print_status 0 "Dockerfile exists"
else
    print_status 1 "Dockerfile missing"
fi

if [ -f "docker-compose.yml" ]; then
    print_status 0 "docker-compose.yml exists"
else
    print_status 1 "docker-compose.yml missing"
fi

echo ""
echo "2. Checking Docker environment..."
echo "================================="

if command -v docker &> /dev/null; then
    print_status 0 "Docker is installed"
    
    if docker compose version &> /dev/null; then
        print_status 0 "Docker Compose v2 available"
    else
        print_status 1 "Docker Compose v2 not available"
    fi
else
    print_status 1 "Docker is not installed"
fi

echo ""
echo "3. Checking container status..."
echo "==============================="

if docker compose ps | grep -q "groupme-bot.*Up"; then
    print_status 0 "GroupMe bot container is running"
    
    # Check health status
    if docker compose ps | grep -q "healthy"; then
        print_status 0 "Container health check passing"
    else
        print_warning "Container health check may be starting or failing"
    fi
    
    # Test webhook endpoint
    if curl -s -I http://localhost:18080/ | grep -q "200 OK"; then
        print_status 0 "Webhook endpoint responding"
    else
        print_status 1 "Webhook endpoint not responding"
    fi
    
else
    print_warning "GroupMe bot container not running"
    echo "Run: docker compose up -d"
fi

echo ""
echo "4. Checking logs for issues..."
echo "=============================="

if docker compose ps | grep -q "groupme-bot"; then
    echo "Recent log entries:"
    docker compose logs --tail 5 2>/dev/null | sed 's/^/  /'
    
    # Check for authentication success
    if docker compose logs 2>/dev/null | grep -q "Service account authentication initialized successfully"; then
        print_status 0 "Google Sheets API authentication working"
    else
        print_status 1 "Google Sheets API authentication may have issues"
    fi
else
    print_warning "No container logs available"
fi

echo ""
echo "5. Environment check..."
echo "======================="

if [ -f ".env" ]; then
    if grep -q "GROUPME_BOT_ID=" .env && ! grep -q "your_bot_id_here" .env; then
        print_status 0 "GROUPME_BOT_ID configured"
    else
        print_status 1 "GROUPME_BOT_ID needs configuration"
    fi
    
    if grep -q "BASE_DOMAIN=" .env && ! grep -q "example.com" .env; then
        print_status 0 "BASE_DOMAIN configured"
    else
        print_warning "BASE_DOMAIN may need configuration"
    fi
fi

echo ""
echo "6. Network connectivity..."
echo "=========================="

DOMAIN=$(grep "BASE_DOMAIN=" .env 2>/dev/null | cut -d'=' -f2)
TEAM=$(grep "TEAM_NAME=" .env 2>/dev/null | cut -d'=' -f2)

if [ ! -z "$DOMAIN" ] && [ ! -z "$TEAM" ] && [ "$DOMAIN" != "example.com" ]; then
    EXTERNAL_URL="https://${TEAM}bot.${DOMAIN}/webhook"
    echo "External URL: $EXTERNAL_URL"
    
    if curl -s -I "$EXTERNAL_URL" --connect-timeout 10 | grep -q "200\|404"; then
        print_status 0 "External URL accessible"
    else
        print_warning "External URL may not be accessible (check DNS/Traefik)"
    fi
fi

echo ""
echo "🎯 Deployment Summary"
echo "==================="
echo ""
echo "Internal webhook: http://localhost:18080/webhook"
if [ ! -z "$EXTERNAL_URL" ]; then
    echo "External webhook: $EXTERNAL_URL"
fi
echo ""
echo "Commands:"
echo "  View logs: docker compose logs -f"
echo "  Restart:   docker compose restart"
echo "  Status:    docker compose ps"
echo "  Stop:      docker compose down"
echo ""
