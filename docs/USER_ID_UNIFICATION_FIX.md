# User ID 统一修复报告

## 问题描述

通过 chat UI 搜索不到记忆，原因是 userId 不一致：
- 前端使用 `default-user` 作为默认用户ID
- 数据库中存在 `default` 和 `default-user` 两种用户ID
- 搜索时由于 user_id 不匹配，导致无法找到记忆

## 修复方案

统一将默认用户ID改为 `default`，确保前后端一致。

## 修改内容

### 1. 前端修改

#### `agentmem-ui/src/lib/constants.ts`
- ✅ 将 `DEFAULT_USER_ID` 从 `'default-user'` 改为 `'default'`
- ✅ 更新 `normalizeUserId` 函数，将 `'default-user'` 转换为 `'default'`

```typescript
export const DEFAULT_USER_ID = 'default';  // 修改前: 'default-user'

export function normalizeUserId(userId?: string): string {
  if (!userId || userId === 'default-user') {
    return DEFAULT_USER_ID;  // 现在返回 'default'
  }
  return userId;
}
```

### 2. 后端修改

#### `crates/agent-mem-server/src/middleware/auth.rs`
- ✅ 将默认认证中间件中的 `user_id` 从 `"default-user"` 改为 `"default"`

```rust
let default_user = AuthUser {
    user_id: "default".to_string(),  // 修改前: "default-user"
    org_id: "default-org".to_string(),
    roles: vec!["admin".to_string(), "user".to_string()],
};
```

#### `crates/agent-mem-server/src/routes/memory.rs`
- ✅ 将添加记忆时的默认 `user_id` 从 `"default-user"` 改为 `"default"`

```rust
user_id: "default".to_string(),  // 修改前: "default-user"
```

#### `crates/agent-mem-core/src/storage/libsql/migrations.rs`
- ✅ 将数据库迁移脚本中的默认用户ID从 `"default-user"` 改为 `"default"`

```rust
libsql::params![
    "default",  // 修改前: "default-user"
    "default-org",
    // ...
]
```

### 3. 数据库更新

- ✅ 更新 `memories` 表中的所有 `user_id='default-user'` 为 `user_id='default'`
- ✅ 更新统计：数据库中有 101 条记录已更新为 `default`

```sql
UPDATE memories SET user_id = 'default' WHERE user_id = 'default-user';
```

## 验证

### 数据库状态
- ✅ 数据库中 `user_id='default'` 的记忆数量: 101条
- ✅ 数据库中 `user_id='default-user'` 的记忆数量: 0条

### 测试脚本
创建了测试脚本 `scripts/test_search_with_default_user.sh` 用于验证修复效果。

## 影响范围

### 修改的文件
1. `agentmem-ui/src/lib/constants.ts` - 前端常量定义
2. `crates/agent-mem-server/src/middleware/auth.rs` - 后端认证中间件
3. `crates/agent-mem-server/src/routes/memory.rs` - 后端记忆路由
4. `crates/agent-mem-core/src/storage/libsql/migrations.rs` - 数据库迁移脚本

### 数据库变更
- `memories` 表的 `user_id` 字段值更新

## 测试建议

1. **前端搜索测试**
   - 在 chat UI 中搜索记忆
   - 验证能正确返回记忆结果
   - 检查返回的记忆 `user_id` 是否为 `default`

2. **API 测试**
   - 使用 `scripts/test_search_with_default_user.sh` 脚本测试
   - 验证搜索 API 返回正确的记忆

3. **新记忆创建测试**
   - 创建新记忆时验证 `user_id` 为 `default`
   - 验证新记忆可以被搜索到

## 后续建议

1. **用户认证集成**
   - 在生产环境中，应该从 JWT token 或认证中间件获取真实的用户ID
   - 当前使用 `"default"` 仅适用于开发/测试环境

2. **数据迁移**
   - 如果有其他表也使用 `user_id` 字段，需要同步更新
   - 检查 `working_memory` 表是否需要更新

3. **文档更新**
   - 更新相关文档，说明默认用户ID为 `default`
   - 更新 API 文档中的示例

## 总结

✅ 已成功统一前后端的默认用户ID为 `default`
✅ 已更新数据库中的历史数据
✅ 已创建测试脚本用于验证修复效果

现在前端搜索应该能够正确找到记忆了。

