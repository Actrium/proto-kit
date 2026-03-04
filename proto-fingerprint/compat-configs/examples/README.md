# Proto-Sign 配置示例

本目录包含不同场景下的 proto-sign 配置示例，帮助用户快速开始使用。

## 📋 配置文件说明

### 🔴 [strict-mode.yaml](./strict-mode.yaml) - 严格模式
- **适用场景**: 公共 API、生产环境、发布库
- **检查范围**: 所有类别 (FILE + PACKAGE + WIRE + WIRE_JSON)
- **特点**: 最严格的检查，确保最大兼容性
- **推荐**: 对外提供的稳定 API

### 🟡 [lenient-mode.yaml](./lenient-mode.yaml) - 宽松模式  
- **适用场景**: 内部 API、开发环境、快速迭代
- **检查范围**: 主要关注 WIRE_JSON + 核心规则
- **特点**: 平衡兼容性和开发效率
- **推荐**: 团队内部使用的 API

### 🔵 [wire-only.yaml](./wire-only.yaml) - 仅线上兼容
- **适用场景**: 只关心序列化兼容性的场景
- **检查范围**: WIRE 类别 (二进制兼容)
- **特点**: 允许源码级变更，只要不影响序列化
- **推荐**: 存储格式、网络协议兼容性

### ⚙️ [specific-rules.yaml](./specific-rules.yaml) - 具体规则
- **适用场景**: 需要精细控制检查项的场景
- **检查范围**: 手动选择的具体规则列表
- **特点**: 最大灵活性，精确控制
- **推荐**: 有特殊需求的复杂项目

## 🚀 使用方法

### 快速开始
```bash
# 复制适合的配置模板
cp compat-configs/examples/lenient-mode.yaml proto-sign.yaml

# 根据项目需求调整配置
vim proto-sign.yaml

# 运行检查
proto-sign breaking --config proto-sign.yaml current/ previous/
```

### 配置格式说明

```yaml
version: v1                    # proto-sign 配置版本
breaking:
  use_categories:              # 使用规则类别 (推荐)
    - FILE                     # 文件级检查
    - PACKAGE                  # 包级检查  
    - WIRE                     # 线上二进制兼容
    - WIRE_JSON               # JSON 兼容
    
  use_rules:                  # 使用具体规则 (精确控制)
    - MESSAGE_NO_DELETE       # 具体规则名
    
  except_rules:               # 排除特定规则
    - FIELD_SAME_NAME         # 不检查字段名一致性
    
  ignore:                     # 忽略特定路径
    - "internal/**"           # 忽略内部包
    - "**/*_test.proto"       # 忽略测试文件
    
  ignore_unstable_packages: true  # 忽略标记为 unstable 的包
```

## 📚 规则类别说明

| 类别 | 说明 | 适用场景 |
|------|------|----------|
| **FILE** | 文件级兼容性 (文件删除、包名等) | 公共 API |
| **PACKAGE** | 包级兼容性 (消息/服务删除等) | 稳定接口 |  
| **WIRE** | 二进制序列化兼容性 | 网络协议 |
| **WIRE_JSON** | JSON 序列化兼容性 | Web API |

## 💡 选择建议

1. **新项目**: 从 `lenient-mode.yaml` 开始
2. **公共库**: 使用 `strict-mode.yaml`  
3. **内部工具**: 使用 `wire-only.yaml`
4. **特殊需求**: 基于 `specific-rules.yaml` 定制

## 🔗 相关资源

- [Proto-Sign 文档](../../../README.md)
- [规则完整列表](../../extracted/EXTRACTION_SUMMARY.md)
- [测试用例](../../extracted/testdata/)