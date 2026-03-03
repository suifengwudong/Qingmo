# 发布指南

## 目录结构

```
release/
├── v0.1.0/
│   ├── text_tool.exe          # 可执行文件
│   ├── README.md              # 发布说明
│   └── text-tool-v0.1.0-windows-x64.zip  # 发布包
├── v0.1.1/
│   └── ...
└── ...
```

## 自动化发布流程

### 准备工作

1. **获取GitHub Token**
   - 访问 https://github.com/settings/tokens
   - 生成新的Personal Access Token
   - 权限：至少需要 `repo` 权限

2. **安装依赖**
   - Rust (cargo)
   - Git
   - PowerShell 5.1+

### 发布步骤

#### 方法1：使用自动化脚本（推荐）

```powershell
# 基本发布
.\publish-release.ps1 -Version "0.1.1" -ApiToken "ghp_your_token_here"

# 跳过构建（如果已构建）
.\publish-release.ps1 -Version "0.1.1" -ApiToken "ghp_xxx" -SkipBuild

# 跳过tag创建
.\publish-release.ps1 -Version "0.1.1" -ApiToken "ghp_xxx" -SkipTag
```

#### 方法2：手动发布

1. **构建release版本**
   ```bash
   cargo build --release
   ```

2. **创建版本目录**
   ```bash
   mkdir release\v0.1.1
   cp target\release\text_tool.exe release\v0.1.1\
   cp README.md release\v0.1.1\
   ```

3. **创建git tag**
   ```bash
   git tag -a v0.1.1 -m "Release v0.1.1"
   git push origin v0.1.1
   ```

4. **创建压缩包**
   ```powershell
   Compress-Archive -Path "release\v0.1.1\*" -DestinationPath "release\v0.1.1\text-tool-v0.1.1-windows-x64.zip"
   ```

5. **在GitHub上创建Release**
   - 访问 https://github.com/suifengwudong/text-tool/releases/new
   - Tag version: v0.1.1
   - Release title: 轻墨小说写作工具 v0.1.1
   - 复制 release\v0.1.1\README.md 的内容作为描述
   - 上传压缩包

## 版本号规范

遵循 [Semantic Versioning](https://semver.org/)：

- **MAJOR.MINOR.PATCH** (例如: 1.2.3)
- **预发布版本**: 1.0.0-alpha.1, 1.0.0-beta.2, 1.0.0-rc.1

## 发布检查清单

- [ ] 代码通过所有测试 (`cargo test`)
- [ ] 代码编译成功 (`cargo build --release`)
- [ ] 更新了README中的版本信息
- [ ] 更新了CHANGELOG（如果有）
- [ ] 提交了所有更改到main分支
- [ ] 准备好了GitHub API Token

## 故障排除

### 构建失败
```bash
# 清理并重新构建
cargo clean
cargo build --release
```

### GitHub API错误
- 检查API Token是否正确
- 确认Token有足够权限
- 检查网络连接

### 脚本权限错误
```powershell
# 设置执行策略
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

## 自动化集成

### GitHub Actions (可选)

可以设置GitHub Actions自动发布：

```yaml
# .github/workflows/release.yml
name: Release
on:
  push:
    tags:
      - 'v*'
jobs:
  release:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo build --release
      - uses: softprops/action-gh-release@v1
        with:
          files: target/release/text_tool.exe
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

这样推送tag时会自动创建release。