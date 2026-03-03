# 使用示例

## 发布新版本

```powershell
# 1. 获取GitHub API Token
# 访问: https://github.com/settings/tokens
# 创建token，权限至少包含 'repo'

# 2. 运行发布脚本
.\publish-release.ps1 -Version "0.1.1" -ApiToken "ghp_your_actual_token_here"

# 脚本会自动：
# - 构建release版本
# - 创建release/v0.1.1/目录
# - 复制可执行文件和文档
# - 创建压缩包
# - 创建git tag并推送
# - 创建GitHub release并上传文件
```

## 手动发布流程

如果不想使用自动化脚本，也可以手动发布：

```powershell
# 1. 构建
cargo build --release

# 2. 创建目录
mkdir release\v0.1.1
cp target\release\text_tool.exe release\v0.1.1\
cp README.md release\v0.1.1\

# 3. 创建压缩包
Compress-Archive -Path "release\v0.1.1\*" -DestinationPath "release\v0.1.1\text-tool-v0.1.1-windows-x64.zip"

# 4. 创建tag
git tag -a v0.1.1 -m "Release v0.1.1"
git push origin v0.1.1

# 5. 在GitHub上手动创建release
# 访问: https://github.com/suifengwudong/text-tool/releases/new
# 上传压缩包并填写说明
```