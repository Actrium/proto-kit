# Buf 测试用例完整移植计划

## 发现的 Buf 测试结构

### 1. 测试数据组织
```
buf/private/bufpkg/bufcheck/testdata/breaking/
├── current/     # 当前版本的 proto 文件
│   ├── breaking_enum_no_delete/
│   ├── breaking_field_no_delete/
│   ├── breaking_message_no_delete/
│   └── ... (60+ 规则测试目录)
└── previous/    # 之前版本的 proto 文件
    ├── breaking_enum_no_delete/
    ├── breaking_field_no_delete/
    ├── breaking_message_no_delete/
    └── ... (对应的规则测试目录)
```

### 2. 测试函数模式
每个规则都有对应的测试函数，例如：
```go
func TestRunBreakingEnumNoDelete(t *testing.T) {
    testBreaking(
        t,
        "breaking_enum_no_delete",
        bufanalysistesting.NewFileAnnotationNoLocation(t, "1.proto", "ENUM_NO_DELETE"),
        bufanalysistesting.NewFileAnnotation(t, "1.proto", 9, 1, 18, 2, "ENUM_NO_DELETE"),
        // ... 期望的错误注解
    )
}
```

### 3. 期望结果格式
- 使用 `FileAnnotation` 描述期望的错误
- 包含文件名、行号、列号、规则 ID
- 支持无位置信息的注解（`NoLocation`）

## 移植策略

### 阶段 1: 复制测试数据 ✅
```bash
# 复制 Buf 的测试数据到 proto-sign
cp -r buf/private/bufpkg/bufcheck/testdata/breaking proto-sign/tests/testdata/
```

### 阶段 2: 创建测试框架
1. **测试助手函数**：创建类似 `testBreaking` 的函数
2. **注解比较**：实现期望结果与实际结果的比较
3. **文件加载**：支持从 `current/` 和 `previous/` 目录加载测试文件

### 阶段 3: 批量生成测试用例
1. **自动生成**：从 Buf 的测试函数自动生成 Rust 测试
2. **规则映射**：确保规则 ID 完全一致
3. **期望结果转换**：将 Go 的 `FileAnnotation` 转换为 Rust 结构

### 阶段 4: 增量实现规则处理器
1. **优先级排序**：按使用频率实现规则处理器
2. **测试驱动**：每实现一个处理器就运行对应测试
3. **回归测试**：确保新实现不破坏已有功能

## 具体实现计划

### 1. 立即行动项
- [ ] 复制 Buf 测试数据
- [ ] 创建测试框架基础设施
- [ ] 实现前 10 个最重要的规则处理器

### 2. 测试数据统计
根据 Buf 测试，发现的规则数量：
- **Enum 规则**: 7 个
- **Field 规则**: 19 个  
- **File 规则**: 21 个
- **Message 规则**: 5 个
- **Service/RPC 规则**: 6 个
- **Package 规则**: 5 个
- **其他规则**: 7 个

**总计**: 70+ 个规则，每个都有完整的测试用例

### 3. 优先实现的规则（基于测试覆盖度）
1. `FIELD_NO_DELETE` - 最常用的字段删除检查
2. `MESSAGE_NO_DELETE` - 消息删除检查
3. `ENUM_NO_DELETE` - 枚举删除检查
4. `SERVICE_NO_DELETE` - 服务删除检查
5. `FIELD_SAME_TYPE` - 字段类型变更检查
6. `ENUM_VALUE_NO_DELETE` - 枚举值删除检查
7. `RPC_NO_DELETE` - RPC 删除检查
8. `FIELD_SAME_NAME` - 字段名称变更检查
9. `FILE_SAME_PACKAGE` - 包名变更检查
10. `FIELD_SAME_CARDINALITY` - 字段基数变更检查

## 技术实现细节

### 测试框架设计
```rust
// 测试助手结构
#[derive(Debug, PartialEq)]
pub struct ExpectedAnnotation {
    pub file: String,
    pub rule_id: String,
    pub start_line: Option<u32>,
    pub start_col: Option<u32>,
    pub end_line: Option<u32>,
    pub end_col: Option<u32>,
}

// 主测试函数
pub fn test_breaking_rule(
    test_name: &str,
    expected_annotations: Vec<ExpectedAnnotation>,
) -> anyhow::Result<()> {
    // 1. 加载 current/ 和 previous/ 目录的文件
    // 2. 运行 breaking change 检测
    // 3. 比较实际结果与期望结果
    // 4. 生成详细的差异报告
}
```

### 自动化测试生成
```rust
// 从 Buf 测试数据自动生成测试用例
#[test] fn test_breaking_enum_no_delete() { /* 自动生成 */ }
#[test] fn test_breaking_field_no_delete() { /* 自动生成 */ }
// ... 70+ 个测试函数
```

## 预期收益

### 1. 完整兼容性保证
- **100% 规则覆盖**：所有 Buf 规则都有对应实现
- **行为一致性**：相同输入产生相同输出
- **边界情况覆盖**：包含 Buf 发现的所有边界情况

### 2. 高质量测试覆盖
- **70+ 规则测试**：每个规则都有专门测试
- **数百个测试用例**：覆盖各种 proto 文件组合
- **回归测试保护**：防止未来修改破坏兼容性

### 3. 开发效率提升
- **测试驱动开发**：先有测试，再实现功能
- **增量开发**：可以逐个规则实现和验证
- **自动化验证**：CI/CD 自动运行所有测试

## 下一步行动

1. **立即开始**：复制 Buf 测试数据
2. **建立框架**：创建测试基础设施
3. **批量实现**：按优先级实现规则处理器
4. **持续集成**：确保所有测试通过

这个移植计划将确保 proto-sign 与 Buf 的完全兼容性，同时提供高质量的测试覆盖。