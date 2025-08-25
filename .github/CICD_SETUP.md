# CI/CD Pipeline Setup Guide

This repository includes a comprehensive CI/CD pipeline that automatically tests, builds, and deploys your GroupMe Pirates Bot.

## Pipeline Overview

### 🔄 Workflow Triggers
- **Push to main branch**: Full pipeline (test → build → deploy)  
- **Pull requests**: Tests and build only (no deployment)
- **Version tags**: Full pipeline with semantic versioning

### 🧪 Testing Pipeline (`test.yml`)
- **Code Quality**: Format checking, clippy linting
- **Unit Tests**: Runs all Cargo tests
- **Security Audit**: Checks for known vulnerabilities
- **Docker Test**: Verifies container builds and starts correctly

### 🏗️ Build Pipeline (`docker-build.yml`)  
- **Multi-platform Build**: AMD64 and ARM64 architectures
- **Docker Hub Push**: Automated image publishing  
- **Artifact Generation**: Creates deployment files
- **Dependency**: Only runs after tests pass

### 🚀 Deployment Pipeline
- **Automatic Deployment**: To production server after successful build
- **Zero-downtime**: Uses rolling update strategy
- **Health Checks**: Verifies deployment success
- **Environment**: Protected production environment

## Required GitHub Secrets

Navigate to your repository → Settings → Secrets and variables → Actions, then add these secrets:

### Docker Hub Authentication
```
DOCKER_USERNAME=your_dockerhub_username
DOCKER_PASSWORD=your_dockerhub_password_or_token
```

### Production Server Deployment (Optional)
```
DEPLOY_HOST=your.server.hostname.com
DEPLOY_USER=your_ssh_username  
DEPLOY_KEY=-----BEGIN OPENSSH PRIVATE KEY-----
your_private_ssh_key_here
-----END OPENSSH PRIVATE KEY-----
DEPLOY_PORT=22                     # Optional: defaults to 22
DEPLOY_PATH=/home/rent/groupme-pirates-bot  # Optional: defaults to this path
```

## Setup Instructions

### 1. Docker Hub Setup
1. Create account at [Docker Hub](https://hub.docker.com)
2. Create repository: `your_username/groupme-pirates-bot`
3. Generate access token: Account Settings → Security → Access Tokens
4. Add `DOCKER_USERNAME` and `DOCKER_PASSWORD` secrets to GitHub

### 2. Production Server Setup (Optional)
If you want automatic deployment to your production server:

1. **Generate SSH Key Pair**:
   ```bash
   ssh-keygen -t ed25519 -C "github-actions-deploy" -f ~/.ssh/github_actions
   ```

2. **Add Public Key to Server**:
   ```bash
   # Copy public key to your server
   ssh-copy-id -i ~/.ssh/github_actions.pub user@your.server.com
   
   # Or manually add to ~/.ssh/authorized_keys on server
   ```

3. **Add Private Key to GitHub Secrets**:
   - Copy contents of `~/.ssh/github_actions` (private key)
   - Add as `DEPLOY_KEY` secret in GitHub

4. **Set Other Deployment Secrets**:
   - `DEPLOY_HOST`: Your server hostname/IP
   - `DEPLOY_USER`: SSH username  
   - `DEPLOY_PORT`: SSH port (optional, defaults to 22)
   - `DEPLOY_PATH`: Path to bot directory (optional)

### 3. Environment Protection (Recommended)
1. Go to repository Settings → Environments
2. Create "production" environment
3. Add protection rules:
   - Required reviewers (for manual approval)
   - Restrict to main branch only
   - Add environment secrets if needed

## Pipeline Behavior

### On Push to Main
1. ✅ **Tests run** (format, lint, test, security audit)
2. 🏗️ **Docker build** (multi-platform, push to Docker Hub)
3. 📦 **Artifacts created** (deployment files)  
4. 🚀 **Auto-deploy** to production (if secrets configured)
5. ✅ **Health check** verifies deployment

### On Pull Request  
1. ✅ **Tests run** (same as above)
2. 🏗️ **Docker build** (test only, no push)
3. ❌ **No deployment**

### Manual Deployment
If you prefer manual deployment, just don't set the `DEPLOY_*` secrets. The pipeline will still:
- Run tests
- Build and push Docker images  
- Create deployment artifacts (downloadable)

Then deploy manually:
```bash
docker compose -f docker-compose.prod.yml pull
docker compose -f docker-compose.prod.yml up -d
```

## Troubleshooting

### Build Failures
- Check the Actions tab in your GitHub repository
- Look at the specific step that failed
- Common issues: formatting, clippy warnings, failed tests

### Deployment Failures  
- Verify SSH connection: `ssh user@host`
- Check server has Docker and docker-compose installed
- Verify paths and permissions
- Check server logs: `docker compose -f docker-compose.prod.yml logs`

### Docker Hub Issues
- Verify username/password are correct
- Check repository exists and is accessible
- Ensure you have push permissions

## Benefits

✅ **Automated Quality**: Every change is tested  
✅ **Fast Feedback**: Know immediately if something breaks  
✅ **Zero-downtime Deploys**: Production stays online during updates  
✅ **Rollback Ready**: Previous images remain tagged  
✅ **Multi-platform**: Works on AMD64 and ARM64 servers  
✅ **Security**: Regular dependency audits  

Your bot will now automatically deploy every time you push to main! 🏴‍☠️🚀
