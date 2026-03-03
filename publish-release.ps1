#Requires -Version 5.1

<#
.SYNOPSIS
    轻墨小说写作工具自动发布脚本

.DESCRIPTION
    自动构建release版本，创建git tag，生成发布包，并创建GitHub release

.PARAMETER Version
    版本号 (例如: 0.1.0)

.PARAMETER ApiToken
    GitHub API token (必需，用于创建release)

.PARAMETER SkipBuild
    跳过构建步骤

.PARAMETER SkipTag
    跳过创建git tag步骤

.EXAMPLE
    .\publish-release.ps1 -Version "0.1.1" -ApiToken "ghp_xxx"

.EXAMPLE
    .\publish-release.ps1 -Version "0.1.1" -ApiToken "ghp_xxx" -SkipBuild
#>

param(
    [Parameter(Mandatory = $true)]
    [string]$Version,

    [Parameter(Mandatory = $true)]
    [string]$ApiToken,

    [switch]$SkipBuild,
    [switch]$SkipTag
)

# 配置
$ProjectName = "text-tool"
$Owner = "suifengwudong"
$Repo = "text-tool"
$TagName = "v$Version"
$ReleaseDir = "release\$TagName"

# 颜色输出函数
function Write-ColorOutput {
    param([string]$Message, [string]$Color = "White")
    Write-Host $Message -ForegroundColor $Color
}

# 检查命令是否存在
function Test-Command {
    param([string]$Command)
    try {
        Get-Command $Command -ErrorAction Stop | Out-Null
        return $true
    }
    catch {
        return $false
    }
}

# 主函数
function Main {
    Write-ColorOutput "🚀 开始发布 $TagName" "Cyan"

    # 检查依赖
    Write-ColorOutput "📋 检查依赖..." "Yellow"
    if (-not (Test-Command "cargo")) {
        throw "未找到cargo，请确保已安装Rust"
    }
    if (-not (Test-Command "git")) {
        throw "未找到git"
    }

    # 检查工作目录状态
    if (-not (Test-Path "Cargo.toml")) {
        throw "请在项目根目录运行此脚本"
    }

    # 构建release版本
    if (-not $SkipBuild) {
        Write-ColorOutput "🔨 构建release版本..." "Yellow"
        & cargo build --release
        if ($LASTEXITCODE -ne 0) {
            throw "构建失败"
        }
        Write-ColorOutput "✅ 构建完成" "Green"
    }

    # 创建release目录
    if (-not (Test-Path $ReleaseDir)) {
        New-Item -ItemType Directory -Path $ReleaseDir -Force | Out-Null
    }

    # 复制可执行文件
    $exePath = "target\release\text_tool.exe"
    if (Test-Path $exePath) {
        Copy-Item $exePath $ReleaseDir
        Write-ColorOutput "📦 复制可执行文件到 $ReleaseDir" "Green"
    } else {
        throw "未找到可执行文件: $exePath"
    }

    # 创建或更新README
    $readmePath = "$ReleaseDir\README.md"
    if (-not (Test-Path $readmePath)) {
        Copy-Item "README.md" $readmePath
        Write-ColorOutput "📝 复制README到release目录" "Green"
    }

    # 创建压缩包
    $zipName = "$ProjectName-$TagName-windows-x64.zip"
    $zipPath = "$ReleaseDir\$zipName"

    Write-ColorOutput "📦 创建发布包..." "Yellow"
    Compress-Archive -Path "$ReleaseDir\*" -DestinationPath $zipPath -Force
    Write-ColorOutput "✅ 发布包创建完成: $zipPath" "Green"

    # 创建git tag
    if (-not $SkipTag) {
        Write-ColorOutput "🏷️ 创建git tag..." "Yellow"

        # 检查tag是否已存在
        $existingTag = & git tag -l $TagName
        if ($existingTag) {
            Write-ColorOutput "⚠️ Tag $TagName 已存在，跳过创建" "Yellow"
        } else {
            & git tag -a $TagName -m "Release $TagName"
            & git push origin $TagName
            Write-ColorOutput "✅ Git tag创建并推送完成" "Green"
        }
    }

    # 创建GitHub release
    Write-ColorOutput "🚀 创建GitHub release..." "Yellow"

    # 读取release说明
    $releaseNotes = Get-Content "$ReleaseDir\README.md" -Raw

    # 构建API请求
    $apiUrl = "https://api.github.com/repos/$Owner/$Repo/releases"
    $headers = @{
        "Authorization" = "token $ApiToken"
        "Accept" = "application/vnd.github.v3+json"
        "Content-Type" = "application/json"
    }

    $body = @{
        tag_name = $TagName
        name = "$ProjectName $TagName"
        body = $releaseNotes
        draft = $false
        prerelease = $Version.Contains("beta") -or $Version.Contains("alpha") -or $Version.Contains("rc")
    } | ConvertTo-Json

    try {
        $response = Invoke-RestMethod -Uri $apiUrl -Method Post -Headers $headers -Body $body
        Write-ColorOutput "✅ GitHub release创建成功" "Green"
        Write-ColorOutput "🔗 Release URL: $($response.html_url)" "Cyan"

        # 上传发布包
        Write-ColorOutput "📤 上传发布包..." "Yellow"
        $uploadUrl = $response.upload_url -replace "\{.*\}", "?name=$zipName"
        $zipContent = Get-Content $zipPath -Encoding Byte

        $uploadHeaders = @{
            "Authorization" = "token $ApiToken"
            "Content-Type" = "application/zip"
        }

        $uploadResponse = Invoke-RestMethod -Uri $uploadUrl -Method Post -Headers $uploadHeaders -Body $zipContent
        Write-ColorOutput "✅ 发布包上传成功" "Green"

    } catch {
        Write-ColorOutput "❌ 创建GitHub release失败: $($_.Exception.Message)" "Red"
        throw
    }

    Write-ColorOutput "`n🎉 发布完成！" "Green"
    Write-ColorOutput "📁 Release文件位置: $ReleaseDir" "Cyan"
    Write-ColorOutput "🏷️ Tag: $TagName" "Cyan"
    Write-ColorOutput "🔗 GitHub Release: $($response.html_url)" "Cyan"
}

# 执行主函数
try {
    Main
} catch {
    Write-ColorOutput "❌ 发布失败: $($_.Exception.Message)" "Red"
    exit 1
}