#[cfg(test)]
mod test_bulk_rules {
    use crate::compat::bulk_rule_registry::{
        get_bulk_rule_count, get_bulk_rule_mapping, verify_bulk_rules,
    };

    #[test]
    fn test_bulk_rule_verification() {
        // Test verification passes
        let result = verify_bulk_rules();
        if let Err(e) = &result {
            println!("验证错误: {e}");
        }
        assert!(result.is_ok(), "Bulk rule verification should pass");

        // Test count is correct (exactly matching Buf's breaking rule count)
        let count = get_bulk_rule_count();
        let expected_count = 69; // Exact 1:1 match with Buf
        assert_eq!(
            count, expected_count,
            "应有{expected_count}个规则，实际有{count}"
        );

        // Test no duplicate rule IDs
        let rules = get_bulk_rule_mapping();
        let mut rule_ids: std::collections::HashSet<&str> = std::collections::HashSet::new();
        for (rule_id, _) in rules {
            assert!(rule_ids.insert(rule_id), "规则ID重复: {rule_id}");
        }

        println!("✅ 规则验证通过! 总计: {count} 规则");
        for (i, (rule_id, _)) in rules.iter().enumerate() {
            println!("  {}. {}", i + 1, rule_id);
        }
    }
}
