# Proto-Sign 兼容性配置管理

这个目录包含 Proto-Sign 兼容性检查的配置文件和管理工具。

## 目录结构

```
compat-configs/
├── extract_buf_configs.sh     # 配置提取脚本
├── examples/                   # 手工维护的配置示例
│   ├── README.md              # 配置使用说明
│   ├── strict-mode.yaml       # 严格模式配置
│   ├── lenient-mode.yaml      # 宽松模式配置
│   ├── wire-only.yaml         # 仅线上兼容配置
│   └── specific-rules.yaml    # 具体规则配置
└── extracted/                  # 自动提取的测试数据
    ├── EXTRACTION_SUMMARY.md  # 提取摘要报告
    ├── main/                  # 转换后的主配置
    └── testdata/              # 测试用例数据
```

## 工具脚本

### `extract_buf_configs.sh`

从上游Buf项目提取配置文件并转换为proto-sign格式。

#### 功能特性

- 🔄 **自动提取**: 从buf项目提取所有相关的YAML配置文件
- 🔧 **格式转换**: 将buf格式转换为proto-sign的简化格式  
- 📂 **分类整理**: 按照用途分类存储（主配置、测试数据、示例）
- 📊 **生成报告**: 自动生成提取摘要和统计信息
- ⚙️ **智能转换**: 自动识别categories vs rules配置

#### 使用方法

```bash
# 基本使用 - 从main分支提取
./compat-configs/extract_buf_configs.sh

# 从特定版本/分支提取
./compat-configs/extract_buf_configs.sh --branch v1.28.1

# 使用环境变量指定分支
BUF_BRANCH=dev ./compat-configs/extract_buf_configs.sh

# 查看帮助
./compat-configs/extract_buf_configs.sh --help

# 仅检查依赖
./compat-configs/extract_buf_configs.sh --check
```

#### 前置条件

1. **git工具**: 脚本会自动克隆buf项目到临时目录
   ```bash
   # 检查git是否安装
   git --version
   ```

2. **网络连接**: 需要从GitHub克隆buf仓库

3. **yq工具** (可选，推荐): 用于更精确的YAML处理
   ```bash
   # macOS
   brew install yq
   
   # Ubuntu/Debian
   sudo apt-get install yq
   
   # 或使用go安装
   go install github.com/mikefarah/yq/v4@latest
   ```

#### 输出结构

```
compat-configs/extracted/
├── main/
│   └── proto-sign-main.yaml    # 转换后的主配置
├── testdata/
│   ├── current/
│   │   └── breaking_*/
│   │       ├── *.proto         # 测试用例proto文件
│   │       ├── buf.yaml        # 原始测试配置
│   │       └── *-protosign.yaml # 转换后的配置
│   └── previous/
│       └── breaking_*/
│           ├── *.proto         # 基线proto文件
│           ├── buf.yaml        # 原始配置
│           └── *-protosign.yaml # 转换后的配置
└── EXTRACTION_SUMMARY.md       # 提取摘要报告
```

#### 转换规则

脚本会自动进行以下转换：

```yaml
# Buf格式
version: v1
breaking:
  use:
    - FILE          # 类别
    - PACKAGE       # 类别
    - FIELD_NO_DELETE  # 具体规则

# 转换为proto-sign格式
version: v1
breaking:
  use_categories:   # 自动识别并分类
    - FILE
    - PACKAGE
  use_rules:        # 具体规则单独列出
    - FIELD_NO_DELETE
```

#### 维护工作流

1. **定期同步**: 当buf项目有重大更新时运行脚本
2. **对比分析**: 检查新增或修改的规则配置
3. **更新适配**: 根据提取结果更新proto-sign的实现
4. **测试验证**: 使用提取的测试配置验证兼容性

#### 示例使用场景

```bash
# 1. 初始设置 - 从最新main分支提取
./compat-configs/extract_buf_configs.sh

# 2. 提取特定发布版本配置
./compat-configs/extract_buf_configs.sh --branch v1.28.1

# 3. 查看提取结果和版本信息
cat compat-configs/extracted/EXTRACTION_SUMMARY.md

# 4. 检查新的规则类型
grep -r "use_categories" compat-configs/extracted/testdata/ | sort -u

# 5. 查找特定规则的配置
find compat-configs/extracted/ -name "*.yaml" -exec grep -l "MESSAGE_NO_DELETE" {} \;

# 6. 对比不同版本的配置变化
git diff HEAD~1 compat-configs/extracted/main/proto-sign-main.yaml

# 7. 定期更新工作流
# 备份当前配置
cp -r compat-configs/extracted compat-configs/extracted.backup
# 提取最新配置
./compat-configs/extract_buf_configs.sh
# 对比变化
diff -r compat-configs/extracted.backup compat-configs/extracted
```

#### 故障排除

- **网络连接问题**: 检查GitHub访问性，考虑使用代理或镜像
- **git命令不存在**: 安装git工具 `apt install git` 或 `brew install git`
- **克隆失败**: 检查网络连接和git配置
- **yq命令不存在**: 脚本会自动降级到手动转换模式
- **权限错误**: 确保脚本有执行权限 `chmod +x extract_buf_configs.sh`
- **临时目录清理**: 脚本会在退出时自动清理临时目录

这个脚本是维护proto-sign与上游buf项目同步的关键工具，建议在CI/CD流程中定期运行。