# 更新日志

## v0.3.1

- 修复腾讯云 DNSPod 在 ACME DNS-01 验证中遇到已存在 `_acme-challenge` TXT 记录时无法继续申请 SSL 证书的问题。
- 新增 DNSPod `DescribeRecordList` 与 `ModifyTXTRecord` 幂等处理流程，记录已存在时会自动查询并更新为本次 ACME 验证 token。
- 优化 DNS TXT 验证记录清理策略：新建记录会删除，复用记录会保留，被临时修改的已有记录会在验证结束后还原。
- 补充 DNSPod 查询、修改、记录已存在错误识别与 ACME 清理策略相关单元测试，降低后续回归风险。
- 同步项目版本号至 `0.3.1`，并让 Release 工作流读取当前 tag 对应的更新日志写入 GitHub Release 页面。
