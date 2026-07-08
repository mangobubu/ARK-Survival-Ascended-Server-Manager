import type { ReactNode } from 'react'
import {
  CloudSyncOutlined,
  DeleteOutlined,
  GlobalOutlined,
  InfoCircleOutlined,
  KeyOutlined,
  LinkOutlined,
  PlusOutlined,
  ReloadOutlined,
  SafetyCertificateOutlined,
  UnlockOutlined,
} from '@ant-design/icons'
import { Button, Form, Input, InputNumber, Popover, Space, Switch, Tag, Typography } from 'antd'
import type { FormInstance } from 'antd/es/form'
import type { GlobalSettings, WebAcmeCertificateStatus, WebSecurityBanRecord } from './types'
import {
  formatBanSource,
  formatBanTime,
  formatRemainingSeconds,
  isValidWebIpWhitelistEntry,
} from './settingsWindowUtils'

const { Text } = Typography

interface SettingsWebAccessSectionProps {
  form: FormInstance<GlobalSettings>
  settings: GlobalSettings
  watchedWebManagementEnabled: boolean
  watchedWebPort: number
  webSettingsDisabled: boolean
  reverseProxySettingsDisabled: boolean
  httpsSettingsDisabled: boolean
  acmeSettingsDisabled: boolean
  webPasswordPlaceholder: string
  tencentSecretKeyPlaceholder: string
  trimmedReverseProxyDomain: string
  reverseProxyUrl: string
  securityBans: WebSecurityBanRecord[]
  securityBansLoading: boolean
  securityBansError: string
  unbanningIp: string | null
  acmeCertificateStatus: WebAcmeCertificateStatus | null
  acmeCertificateStatusLoading: boolean
  acmeCertificateStatusError: string
  directoryPicker: (field: 'webReverseProxyOpenRestyPath', label: string, disabled?: boolean) => ReactNode
  onLoadSecurityBans: () => void
  onUnbanSecurityIp: (ip: string) => void
  onRefreshAcmeCertificateStatus: () => void
}

function formatUnixTime(value: number | null | undefined) {
  if (!value) return '未知'
  return new Date(value * 1000).toLocaleString()
}

function formatDaysRemaining(expiresAtUnix: number) {
  const days = Math.ceil((expiresAtUnix * 1000 - Date.now()) / 86_400_000)
  if (days < 0) return `已过期 ${Math.abs(days)} 天`
  if (days === 0) return '今天到期'
  return `剩余 ${days} 天`
}

export default function SettingsWebAccessSection({
  form,
  settings,
  watchedWebManagementEnabled,
  watchedWebPort,
  webSettingsDisabled,
  reverseProxySettingsDisabled,
  httpsSettingsDisabled,
  acmeSettingsDisabled,
  webPasswordPlaceholder,
  tencentSecretKeyPlaceholder,
  trimmedReverseProxyDomain,
  reverseProxyUrl,
  securityBans,
  securityBansLoading,
  securityBansError,
  unbanningIp,
  acmeCertificateStatus,
  acmeCertificateStatusLoading,
  acmeCertificateStatusError,
  directoryPicker,
  onLoadSecurityBans,
  onUnbanSecurityIp,
  onRefreshAcmeCertificateStatus,
}: SettingsWebAccessSectionProps) {
  return (
    <section className="settings-card settings-card--full">
      <div className="settings-card__heading settings-card__heading--compact">
        <div className="settings-card__icon"><GlobalOutlined /></div>
        <div><h2>Web 访问</h2><p>浏览器版本入口与监听端口</p></div>
      </div>
      <div className="settings-card__body settings-card__body--two-column">
        <div className="settings-toggle-row settings-toggle-row--wide">
          <div>
            <strong>启用 Web 管理</strong>
            <span>关闭时不启动浏览器管理端，并锁定下面的 Web 访问、登录与反代字段；开启后点击保存才会启动。</span>
          </div>
          <Form.Item name="webManagementEnabled" valuePropName="checked">
            <Switch
              onChange={(enabled) => {
                if (!enabled) {
                  form.setFieldValue('webReverseProxyEnabled', false)
                  form.setFieldValue('webHttpsEnabled', false)
                  form.setFieldValue('webAcmeAutoIssueEnabled', false)
                }
              }}
            />
          </Form.Item>
        </div>
        <Form.Item
          label="Web 访问端口"
          name="webServerPort"
          tooltip="Web 管理启用后监听该端口；修改保存后会即时重启 Web 管理服务"
          rules={[
            { required: true, message: '请输入 Web 访问端口' },
            { type: 'number', min: 1024, max: 65535, message: '端口必须在 1024-65535 之间' },
          ]}
        >
          <InputNumber disabled={webSettingsDisabled} min={1024} max={65535} addonBefore="127.0.0.1:" style={{ width: '100%' }} />
        </Form.Item>
        <div className="settings-web-port-note">
          <Text type="secondary">{watchedWebManagementEnabled ? '保存后访问地址：' : '启用并保存后访问地址：'}</Text>
          <Text code copyable={watchedWebManagementEnabled ? { text: `http://127.0.0.1:${watchedWebPort}` } : false}>http://127.0.0.1:{watchedWebPort}</Text>
          <Text type="secondary">关闭时不会监听端口；开启或修改端口后，点击保存才会启动或切换 Web 管理服务。</Text>
        </div>
        <div className="settings-toggle-row settings-toggle-row--wide">
          <div>
            <strong>
              启用域名反向代理
              <Popover
                content={(
                  <div className="settings-popover-note">
                    <p>使用 OpenResty 在公开端口监听域名，再转发到本机 127.0.0.1 的应用 Web 端口。</p>
                    <p>Lua 安全网关会处理动态封禁、异常 UA、危险请求体、频率与路径组合风险。</p>
                    <p>需要你自行把域名 DNS 解析到这台 Windows 主机；本功能不会修改公网 DNS。</p>
                    <p>启用后，Web 后端会拒绝非“域名:公开端口”的 Host 请求。</p>
                  </div>
                )}
                placement="right"
                title="域名访问说明"
              >
                <InfoCircleOutlined className="settings-inline-help" />
              </Popover>
            </strong>
            <span>使用 OpenResty for Windows，便于后续接入 Redis 黑白名单与更复杂 Lua 风控</span>
          </div>
          <Form.Item name="webReverseProxyEnabled" valuePropName="checked">
            <Switch
              disabled={webSettingsDisabled}
              onChange={(enabled) => {
                if (!enabled) {
                  form.setFieldValue('webHttpsEnabled', false)
                  form.setFieldValue('webAcmeAutoIssueEnabled', false)
                }
              }}
            />
          </Form.Item>
        </div>
        <Form.Item
          label="公开访问域名"
          name="webReverseProxyDomain"
          tooltip="只填写域名主机名，例如 ark.example.com；不要填写 http://、端口或路径"
          rules={[
            ({ getFieldValue }) => ({
              validator(_, value) {
                if (!getFieldValue('webManagementEnabled') || !getFieldValue('webReverseProxyEnabled')) return Promise.resolve()
                const domain = String(value ?? '').trim()
                if (!domain) return Promise.reject(new Error('启用反向代理时必须填写访问域名'))
                if (/(:\/\/|[\\/:*])/.test(domain)) return Promise.reject(new Error('域名不能包含协议、端口、路径或通配符'))
                if (/^(localhost|(\d{1,3}\.){3}\d{1,3})$/i.test(domain)) return Promise.reject(new Error('请填写真实域名，不要填写 localhost 或 IP 地址'))
                const labelPattern = /^[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?$/i
                if (!domain.replace(/\.$/, '').split('.').every((label) => labelPattern.test(label))) {
                  return Promise.reject(new Error('域名格式无效，只支持字母、数字、短横线和点号'))
                }
                return Promise.resolve()
              },
            }),
          ]}
        >
          <Input disabled={reverseProxySettingsDisabled} prefix={<LinkOutlined />} placeholder="例如：ark.example.com" />
        </Form.Item>
        <Form.Item
          label="反代公开端口"
          name="webReverseProxyPort"
          tooltip="OpenResty 对外监听的 HTTP 端口；不能与应用 Web 内部端口相同"
          rules={[
            ({ getFieldValue }) => ({
              validator(_, value) {
                if (!getFieldValue('webManagementEnabled') || !getFieldValue('webReverseProxyEnabled')) return Promise.resolve()
                if (typeof value !== 'number') return Promise.reject(new Error('请输入反代公开端口'))
                if (value < 1 || value > 65535) return Promise.reject(new Error('端口必须在 1-65535 之间'))
                if (value === getFieldValue('webServerPort')) return Promise.reject(new Error('公开端口不能与应用 Web 内部端口相同'))
                return Promise.resolve()
              },
            }),
          ]}
        >
          <InputNumber disabled={reverseProxySettingsDisabled} min={1} max={65535} addonBefore="0.0.0.0:" style={{ width: '100%' }} />
        </Form.Item>
        <Form.Item
          label="OpenResty 安装目录"
          name="webReverseProxyOpenRestyPath"
          tooltip="填写 OpenResty 解压目录或其中的 nginx.exe 路径；应用会使用独立前缀目录生成专用配置和 Lua 安全脚本"
          rules={[
            ({ getFieldValue }) => ({
              validator(_, value) {
                if (!getFieldValue('webManagementEnabled') || !getFieldValue('webReverseProxyEnabled')) return Promise.resolve()
                if (String(value ?? '').trim()) return Promise.resolve()
                return Promise.reject(new Error('启用反向代理时必须填写 OpenResty 安装目录或 nginx.exe 路径'))
              },
            }),
          ]}
        >
          <Input
            disabled={reverseProxySettingsDisabled}
            prefix={<CloudSyncOutlined />}
            suffix={directoryPicker('webReverseProxyOpenRestyPath', 'OpenResty 安装目录', reverseProxySettingsDisabled)}
            placeholder="例如：C:\\openresty-1.25.3.2-win64 或 C:\\openresty-1.25.3.2-win64\\nginx.exe"
          />
        </Form.Item>
        <div className="settings-toggle-row settings-toggle-row--wide">
          <div>
            <strong>启用 HTTPS</strong>
            <span>启用后 OpenResty 使用 HTTPS 公开访问，并自动写入安全响应头；证书由下方 ACME 配置申请或续期。</span>
          </div>
          <Form.Item name="webHttpsEnabled" valuePropName="checked">
            <Switch
              disabled={reverseProxySettingsDisabled}
              onChange={(enabled) => {
                if (!enabled) form.setFieldValue('webAcmeAutoIssueEnabled', false)
              }}
            />
          </Form.Item>
        </div>
        <div className="settings-toggle-row settings-toggle-row--wide">
          <div>
            <strong>Let's Encrypt 自动申请/续期</strong>
            <span>使用 DNS-01 验证，RSA2048 证书密钥，申请时自动创建腾讯云 DNS TXT 记录。</span>
          </div>
          <Form.Item name="webAcmeAutoIssueEnabled" valuePropName="checked">
            <Switch disabled={httpsSettingsDisabled} />
          </Form.Item>
        </div>
        <div className="settings-acme-status-card">
          <div className="settings-acme-status-card__head">
            <div>
              <strong>证书状态</strong>
              <span>已存在证书时显示证书到期时间与下一次自动续期时间</span>
            </div>
            <Button icon={<ReloadOutlined />} loading={acmeCertificateStatusLoading} onClick={onRefreshAcmeCertificateStatus} size="small">
              刷新
            </Button>
          </div>
          {acmeCertificateStatusError ? (
            <Text type="secondary">证书状态暂不可用：{acmeCertificateStatusError}</Text>
          ) : acmeCertificateStatus ? (
            <div className="settings-acme-status-grid">
              <div>
                <Text type="secondary">域名</Text>
                <Text code>{acmeCertificateStatus.domain}</Text>
              </div>
              <div>
                <Text type="secondary">证书到期时间</Text>
                <Space wrap size={6}>
                  <Text>{formatUnixTime(acmeCertificateStatus.expiresAtUnix)}</Text>
                  <Tag color={acmeCertificateStatus.expiresAtUnix * 1000 <= Date.now() ? 'red' : 'green'}>
                    {formatDaysRemaining(acmeCertificateStatus.expiresAtUnix)}
                  </Tag>
                </Space>
              </div>
              <div>
                <Text type="secondary">下次续期时间</Text>
                <Text>{formatUnixTime(acmeCertificateStatus.renewAfterUnix)}</Text>
              </div>
              <div>
                <Text type="secondary">签发时间</Text>
                <Text>{formatUnixTime(acmeCertificateStatus.issuedAtUnix)}</Text>
              </div>
            </div>
          ) : (
            <Text type="secondary">暂无已签发证书。保存并成功完成 Let's Encrypt 自动申请后，会在这里显示到期与续期时间。</Text>
          )}
        </div>
        <Form.Item
          label="ACME 目录地址"
          name="webAcmeDirectoryUrl"
          tooltip="当前固定使用 Let's Encrypt 正式环境 ACME v2 目录"
          rules={[
            ({ getFieldValue }) => ({
              validator(_, value) {
                if (!getFieldValue('webAcmeAutoIssueEnabled')) return Promise.resolve()
                if (String(value ?? '').trim() === 'https://acme-v02.api.letsencrypt.org/directory') return Promise.resolve()
                return Promise.reject(new Error('当前仅支持 Let’s Encrypt 正式环境目录地址'))
              },
            }),
          ]}
        >
          <Input disabled={acmeSettingsDisabled} prefix={<SafetyCertificateOutlined />} />
        </Form.Item>
        <Form.Item
          label="ACME 账户邮箱"
          name="webAcmeAccountEmail"
          tooltip="用于注册 Let's Encrypt ACME 账户和接收证书到期提醒"
          rules={[
            ({ getFieldValue }) => ({
              validator(_, value) {
                if (!getFieldValue('webAcmeAutoIssueEnabled')) return Promise.resolve()
                const email = String(value ?? '').trim()
                if (/^[^@\s]+@[^@\s]+\.[^@\s]+$/.test(email)) return Promise.resolve()
                return Promise.reject(new Error('请输入有效的 ACME 账户邮箱'))
              },
            }),
          ]}
        >
          <Input disabled={acmeSettingsDisabled} placeholder="例如：admin@example.com" />
        </Form.Item>
        <Form.Item
          label="腾讯云 Secret ID"
          name="webAcmeTencentSecretId"
          tooltip="用于调用腾讯云 DNSPod API 自动创建 _acme-challenge TXT 记录"
          rules={[
            ({ getFieldValue }) => ({
              validator(_, value) {
                if (!getFieldValue('webAcmeAutoIssueEnabled')) return Promise.resolve()
                if (String(value ?? '').trim()) return Promise.resolve()
                return Promise.reject(new Error('启用 ACME 自动申请时必须填写腾讯云 Secret ID'))
              },
            }),
          ]}
        >
          <Input disabled={acmeSettingsDisabled} prefix={<KeyOutlined />} placeholder="AKID..." />
        </Form.Item>
        <Form.Item
          label="腾讯云 Secret Key"
          name="webAcmeTencentSecretKey"
          tooltip="保存后不会回显；留空会保留当前已保存的 Secret Key"
          rules={[
            ({ getFieldValue }) => ({
              validator(_, value) {
                if (!getFieldValue('webAcmeAutoIssueEnabled')) return Promise.resolve()
                if (String(value ?? '').trim() || settings.webAcmeTencentSecretKeyConfigured) return Promise.resolve()
                return Promise.reject(new Error('启用 ACME 自动申请时必须填写腾讯云 Secret Key'))
              },
            }),
          ]}
        >
          <Input.Password disabled={acmeSettingsDisabled} prefix={<KeyOutlined />} placeholder={tencentSecretKeyPlaceholder} />
        </Form.Item>
        <div className="settings-web-port-note">
          <Text type="secondary">公网反代地址：</Text>
          <Text code copyable={trimmedReverseProxyDomain ? { text: reverseProxyUrl } : false}>{reverseProxyUrl}</Text>
          <Text type="secondary">HTTPS 证书文件将保存在应用数据目录的 web-reverse-proxy/certs 下，并由 OpenResty 专用配置引用。</Text>
        </div>
        <Form.Item
          label="登录失败封禁阈值"
          name="webLoginFailureBanThreshold"
          tooltip="同一 IP 在登录接口返回 401 的累计次数达到阈值后，OpenResty Lua 会临时封禁该 IP"
          rules={[
            { required: true, message: '请输入登录失败封禁阈值' },
            { type: 'number', min: 1, max: 100, message: '登录失败封禁阈值必须在 1-100 次之间' },
          ]}
        >
          <InputNumber disabled={reverseProxySettingsDisabled} min={1} max={100} addonAfter="次" style={{ width: '100%' }} />
        </Form.Item>
        <Form.Item
          label="登录失败封禁时长"
          name="webLoginFailureBanSeconds"
          tooltip="同一 IP 触发登录失败封禁后的封禁秒数；默认 1800 秒"
          rules={[
            { required: true, message: '请输入登录失败封禁时长' },
            { type: 'number', min: 1, max: 86400, message: '登录失败封禁时长必须在 1-86400 秒之间' },
          ]}
        >
          <InputNumber disabled={reverseProxySettingsDisabled} min={1} max={86400} addonAfter="秒" style={{ width: '100%' }} />
        </Form.Item>
        <Form.Item
          label="验证码字符池"
          name="webCaptchaCharset"
          tooltip="登录失败后生成字符串验证码时使用的候选字符；会写入 config.toml，可手动编辑"
          rules={[
            { required: true, message: '请输入验证码字符池' },
            { max: 128, message: '验证码字符池不能超过 128 个字符' },
            {
              validator(_, value) {
                const visibleLength = String(value ?? '').trim().replace(/\s/g, '').length
                if (visibleLength < 2) return Promise.reject(new Error('验证码字符池至少需要 2 个可见字符'))
                return Promise.resolve()
              },
            },
          ]}
        >
          <Input disabled={webSettingsDisabled} placeholder="例如：ABCDEFGHJKLMNPQRSTUVWXYZ23456789" />
        </Form.Item>
        <Form.Item
          label="验证码字符数量"
          name="webCaptchaLength"
          tooltip="每张验证码随机抽取的字符数量；会写入 config.toml"
          rules={[
            { required: true, message: '请输入验证码字符数量' },
            { type: 'number', min: 1, max: 12, message: '验证码字符数量必须在 1-12 之间' },
          ]}
        >
          <InputNumber disabled={webSettingsDisabled} min={1} max={12} addonAfter="个字符" style={{ width: '100%' }} />
        </Form.Item>
        <Form.Item
          label="验证码字体大小"
          name="webCaptchaFontSize"
          tooltip="SVG 验证码文字的字号；会写入 config.toml"
          rules={[
            { required: true, message: '请输入验证码字体大小' },
            { type: 'number', min: 18, max: 56, message: '验证码字体大小必须在 18-56 之间' },
          ]}
        >
          <InputNumber disabled={webSettingsDisabled} min={18} max={56} addonAfter="px" style={{ width: '100%' }} />
        </Form.Item>
        <Form.Item
          label="验证码杂点数量"
          name="webCaptchaNoisePoints"
          tooltip="SVG 验证码中的随机杂点数量；0 表示不添加杂点，会写入 config.toml"
          rules={[
            { required: true, message: '请输入验证码杂点数量' },
            { type: 'number', min: 0, max: 120, message: '验证码杂点数量必须在 0-120 之间' },
          ]}
        >
          <InputNumber disabled={webSettingsDisabled} min={0} max={120} addonAfter="个" style={{ width: '100%' }} />
        </Form.Item>
        <div className="settings-web-captcha-note">
          <Text type="secondary">验证码策略：</Text>
          <Text>首次 Web 登录失败后，同一访问来源在后续一小时内登录都必须先通过字符串验证码；验证码参数会持久化到 <code>config.toml</code>。</Text>
        </div>
        <div className="settings-web-whitelist-editor">
          <div className="settings-web-whitelist-editor__head">
            <div>
              <strong>准入 IP 白名单</strong>
              <span>支持 CN_MAINLAND、单个 IPv4 或 IPv4 CIDR，并可填写分组与备注</span>
            </div>
            <Popover
              content={(
                <div className="settings-popover-note">
                  <p>白名单条目会持久化保存到全局设置；OpenResty 运行目录下的 CIDR 文件只是根据这些设置派生生成。</p>
                  <p>如果手动修改运行目录里的 CIDR 文件，下次应用设置或重启反代时会被全局设置覆盖。</p>
                  <p>内网与环回地址始终放行，避免本机管理被误拦截。</p>
                </div>
              )}
              placement="right"
              title="白名单持久化说明"
            >
              <InfoCircleOutlined className="settings-inline-help" />
            </Popover>
          </div>
          <Form.List name="webIpWhitelist">
            {(fields, { add, remove }) => (
              <div className="settings-web-whitelist-list">
                {fields.map((field) => (
                  <div className="settings-web-whitelist-row" key={field.key}>
                    <Form.Item
                      {...field}
                      label="IP / CIDR"
                      name={[field.name, 'value']}
                      rules={[
                        {
                          validator(_, value) {
                            const entry = String(value ?? '').trim()
                            if (!entry) return Promise.reject(new Error('请输入白名单 IP、CIDR 或 CN_MAINLAND'))
                            if (!isValidWebIpWhitelistEntry(entry)) {
                              return Promise.reject(new Error('仅支持 CN_MAINLAND、IPv4 或 IPv4 CIDR'))
                            }
                            return Promise.resolve()
                          },
                        },
                      ]}
                    >
                      <Input disabled={reverseProxySettingsDisabled} placeholder="CN_MAINLAND 或 203.0.113.0/24" />
                    </Form.Item>
                    <Form.Item
                      {...field}
                      label="分组"
                      name={[field.name, 'group']}
                      rules={[{ max: 32, message: '分组不能超过 32 个字符' }]}
                    >
                      <Input disabled={reverseProxySettingsDisabled} placeholder="例如：大陆默认 / 运维 / 临时" />
                    </Form.Item>
                    <Form.Item
                      {...field}
                      label="备注"
                      name={[field.name, 'note']}
                      rules={[{ max: 120, message: '备注不能超过 120 个字符' }]}
                    >
                      <Input disabled={reverseProxySettingsDisabled} placeholder="例如：办公室固定出口 IP" />
                    </Form.Item>
                    <Button
                      danger
                      disabled={reverseProxySettingsDisabled || fields.length <= 1}
                      icon={<DeleteOutlined />}
                      onClick={() => remove(field.name)}
                    >
                      删除
                    </Button>
                  </div>
                ))}
                <Button
                  disabled={reverseProxySettingsDisabled}
                  icon={<PlusOutlined />}
                  onClick={() => add({ value: '', group: '', note: '' })}
                >
                  添加白名单条目
                </Button>
              </div>
            )}
          </Form.List>
        </div>
        <div className="settings-web-proxy-note">
          <Text type="secondary">域名入口：</Text>
          <Text code copyable={trimmedReverseProxyDomain ? { text: reverseProxyUrl } : false}>{reverseProxyUrl}</Text>
          <Text type="secondary">保存后会生成独立 OpenResty 配置与 Lua 安全脚本，未知 Host 默认 403；未命中 IP 白名单的公网 IP 默认拦截。</Text>
        </div>
        <div className="settings-web-ban-panel">
          <div className="settings-web-ban-panel__head">
            <div>
              <strong>登录失败封禁记录</strong>
              <span>读取当前 OpenResty Lua 共享内存中的动态封禁，可手动解封 IP</span>
            </div>
            <Button disabled={reverseProxySettingsDisabled} icon={<ReloadOutlined />} loading={securityBansLoading} onClick={onLoadSecurityBans} size="small">
              刷新
            </Button>
          </div>
          {securityBansError ? (
            <Text type="secondary">封禁记录暂不可用：{securityBansError}</Text>
          ) : securityBans.length === 0 ? (
            <Text type="secondary">暂无活动封禁记录。启用 OpenResty 反代并触发登录失败封禁后会在这里显示。</Text>
          ) : (
            <div className="settings-web-ban-list">
              {securityBans.map((record) => (
                <div className="settings-web-ban-row" key={`${record.ip}-${record.bannedAtMs}`}>
                  <div>
                    <Space wrap size={6}>
                      <Text code>{record.ip}</Text>
                      <Tag color={record.source === 'login' ? 'red' : 'orange'}>{formatBanSource(record.source)}</Tag>
                      <Text type="secondary">剩余 {formatRemainingSeconds(record.remainingSeconds)}</Text>
                    </Space>
                    <p>{record.reason} · 封禁时间：{formatBanTime(record.bannedAtMs)}</p>
                  </div>
                  <Button
                    icon={<UnlockOutlined />}
                    disabled={reverseProxySettingsDisabled}
                    loading={unbanningIp === record.ip}
                    onClick={() => onUnbanSecurityIp(record.ip)}
                    size="small"
                  >
                    解封
                  </Button>
                </div>
              ))}
            </div>
          )}
        </div>
        <Form.Item
          label="Web 管理员账号"
          name="webAdminUsername"
          tooltip="仅用于浏览器 Web 端登录；桌面端不需要登录"
          rules={[
            { max: 64, message: '账号不能超过 64 个字符' },
            ({ getFieldValue }) => ({
              validator(_, value) {
                if (!getFieldValue('webAdminPassword') || String(value ?? '').trim()) return Promise.resolve()
                if (!getFieldValue('webManagementEnabled')) return Promise.resolve()
                return Promise.reject(new Error('设置密码时管理员账号不能为空'))
              },
            }),
          ]}
        >
          <Input disabled={webSettingsDisabled} prefix={<SafetyCertificateOutlined />} placeholder="例如：admin" />
        </Form.Item>
        <Form.Item
          label="Web 管理员密码"
          name="webAdminPassword"
          tooltip="后端不会回传明文密码；已设置密码时留空保存会保留旧密码"
          rules={[{ max: 128, message: '密码不能超过 128 个字符' }]}
        >
          <Input.Password disabled={webSettingsDisabled} autoComplete="new-password" placeholder={webPasswordPlaceholder} />
        </Form.Item>
        <div className="settings-web-auth-note">
          <Text type="secondary">登录页提示：</Text>
          <Text>Web 端会提示用户回到桌面端「全局设置 → Web 访问」部署管理员账号和密码。</Text>
          {settings.webAdminPasswordConfigured && (
            <Text type="secondary">当前已配置管理员密码。密码输入框留空时只保存其他设置，不会覆盖旧密码。</Text>
          )}
        </div>
      </div>
    </section>
  )
}
