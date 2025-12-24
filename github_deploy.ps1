# Matrix IPTV - GitHub Deployment Automator (Simplified Parser Fix)

Write-Host "🚀 Preparing Matrix IPTV for GitHub..." -ForegroundColor Cyan

# 1. Check for Git
if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Git is not installed. Please install it from git-scm.com first." -ForegroundColor Red
    exit
}

# 2. Check Git Identity
$gitName = git config user.name
$gitEmail = git config user.email

if (-not $gitName -or -not $gitEmail) {
    Write-Host "👤 Git personality not set. Let's fix that!" -ForegroundColor Yellow
    if (-not $gitName) {
        $gitNameInput = Read-Host "👉 Enter your name"
        git config --global user.name "$gitNameInput"
    }
    if (-not $gitEmail) {
        $gitEmailInput = Read-Host "👉 Enter your email"
        git config --global user.email "$gitEmailInput"
    }
    Write-Host "✅ Identity saved!" -ForegroundColor Green
}

# 3. Initialize and Setup Branch
if (-not (Test-Path ".git")) {
    Write-Host "📂 Initializing local repository..." -ForegroundColor Gray
    git init
}

# Force rename current branch to main
git branch -M main
Write-Host "📍 Branch set to main." -ForegroundColor Gray

# 4. Stage and Commit
Write-Host "📝 Staging files..." -ForegroundColor Gray
git add .

# Check if we have a commit yet
git rev-parse HEAD 2>$null
if (-not $?) {
    Write-Host "💾 Creating your very first commit..." -ForegroundColor Yellow
    git commit -m "Initial commit: Matrix IPTV System"
}
else {
    $status = git status --porcelain
    if ($status) {
        Write-Host "💾 Committing new changes..." -ForegroundColor Yellow
        git commit -m "Update: Matrix IPTV System"
    }
    else {
        Write-Host "✅ Everything is already up to date." -ForegroundColor Green
    }
}

# 5. Setup Remote
$remote = git remote get-url origin -ErrorAction SilentlyContinue
if (-not $remote) {
    Write-Host ""
    Write-Host "⚠️ ACTION REQUIRED: Link your GitHub account." -ForegroundColor Yellow
    Write-Host "1. Create a repository on GitHub named matrix-iptv" -ForegroundColor White
    
    $repoUrlInput = Read-Host "📋 Paste your GitHub URL here"
    
    if ($repoUrlInput) {
        git remote add origin $repoUrlInput
        Write-Host "✅ Connected to GitHub!" -ForegroundColor Green
    }
    else {
        Write-Host "❌ No URL provided. Deployment cancelled." -ForegroundColor Red
        exit
    }
}

# 6. Push to GitHub
Write-Host "🛰️ Uploading to GitHub..." -ForegroundColor Blue
git push -u origin main

if ($?) {
    Write-Host ""
    Write-Host "🎉 SUCCESS! Your code is now live on GitHub." -ForegroundColor Green
}
else {
    Write-Host "❌ Upload failed." -ForegroundColor Red
}

Write-Host "Press any key to finish..."
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
