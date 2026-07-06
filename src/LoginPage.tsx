import { useCallback, useEffect, useMemo, useState } from 'react'
import { LockOutlined, ReloadOutlined, SafetyCertificateOutlined, UserOutlined } from '@ant-design/icons'
import { Alert, Button, Form, Input, Typography, message } from 'antd'
import {
  getWebCaptcha,
  getWebAuthStatus,
  loginWeb,
  type WebCaptcha,
  type WebAuthStatus,
} from './backendApi'

const { Text, Title } = Typography

interface LoginPageProps {
  onAuthenticated: () => void
}

interface LoginFormValues {
  username: string
  password: string
  captchaAnswer?: string
}

export default function LoginPage({ onAuthenticated }: LoginPageProps) {
  const [form] = Form.useForm<LoginFormValues>()
  const [messageApi, contextHolder] = message.useMessage()
  const [authStatus, setAuthStatus] = useState<WebAuthStatus | null>(null)
  const [captcha, setCaptcha] = useState<WebCaptcha | null>(null)
  const [loadingStatus, setLoadingStatus] = useState(true)
  const [loadingCaptcha, setLoadingCaptcha] = useState(false)
  const [submitting, setSubmitting] = useState(false)
  const captchaRequired = authStatus?.captchaRequired === true
  const captchaImageUrl = useMemo(() => {
    if (!captcha?.imageSvg) return ''
    return `data:image/svg+xml;charset=utf-8,${encodeURIComponent(captcha.imageSvg)}`
  }, [captcha])

  const refreshCaptcha = useCallback(async () => {
    setLoadingCaptcha(true)
    try {
      const next = await getWebCaptcha()
      setCaptcha(next.required ? next : null)
      form.setFieldValue('captchaAnswer', '')
    } catch (error) {
      messageApi.error(`无法刷新验证码：${String(error)}`)
      setCaptcha(null)
    } finally {
      setLoadingCaptcha(false)
    }
  }, [form, messageApi])

  const refreshAuthStatus = useCallback(async () => {
    const status = await getWebAuthStatus()
    setAuthStatus(status)
    if (!status.captchaRequired) {
      setCaptcha(null)
      form.setFieldValue('captchaAnswer', '')
    }
    return status
  }, [form])

  useEffect(() => {
    void refreshAuthStatus()
      .catch((error) => {
        messageApi.error(`无法读取 Web 登录配置：${String(error)}`)
      })
      .finally(() => setLoadingStatus(false))
  }, [messageApi, refreshAuthStatus])

  useEffect(() => {
    if (!captchaRequired || loadingStatus || captcha) return
    void refreshCaptcha()
  }, [captcha, captchaRequired, loadingStatus, refreshCaptcha])

  const handleFinish = async (values: LoginFormValues) => {
    setSubmitting(true)
    try {
      await loginWeb(
        values.username,
        values.password,
        captchaRequired
          ? {
              token: captcha?.token ?? '',
              answer: values.captchaAnswer ?? '',
            }
          : undefined,
      )
      messageApi.success('登录成功，正在进入管理控制台')
      onAuthenticated()
    } catch (error) {
      messageApi.error(String(error instanceof Error ? error.message : error))
      const status = await refreshAuthStatus().catch(() => null)
      if (status?.captchaRequired) await refreshCaptcha()
    } finally {
      setSubmitting(false)
    }
  }

  return (
    <div className="web-login-page">
      {contextHolder}
      <div className="web-login-page__aurora" />
      <section className="web-login-hero">
        <div className="web-login-hero__badge">ARK: Survival Ascended · Web Console</div>
        <h1>方舟服务器远程管理入口</h1>
        <p>通过桌面端部署的管理员账号进入 Web 控制台，集中管理实例、配置、日志、备份与 RCON 操作。</p>
        <div className="web-login-hero__chips">
          <span>本机 Web 服务</span>
          <span>全操作鉴权</span>
          <span>桌面端统一配置</span>
        </div>
      </section>

      <section className="web-login-card" aria-label="Web 管理员登录">
        <div className="web-login-card__logo">
          <img src="/app-icon.png" alt="方舟进化飞升服务器管理器 LOGO" />
        </div>
        <div className="web-login-card__title">
          <Title level={3}>管理员登录</Title>
          <Text>Web 端需要使用桌面端全局设置里部署的管理员账号和密码。</Text>
        </div>

        {authStatus && !authStatus.configured ? (
          <Alert
            className="web-login-alert"
            type="warning"
            showIcon
            message="尚未部署 Web 管理员账号和密码"
            description="请回到桌面端，打开「全局设置 → Web 访问」，填写管理员账号与密码并保存后，再刷新此页面登录。"
          />
        ) : (
          <Alert
            className="web-login-alert"
            type="info"
            showIcon
            message="账号来源提示"
            description="如果忘记账号或密码，请在桌面端的全局设置中重新部署 Web 管理员账号和密码。"
          />
        )}

        <Form<LoginFormValues>
          className="web-login-form"
          form={form}
          layout="vertical"
          requiredMark={false}
          onFinish={handleFinish}
          initialValues={{ username: 'admin' }}
        >
          <Form.Item
            label="管理员账号"
            name="username"
            rules={[{ required: true, message: '请输入桌面端配置的管理员账号' }]}
          >
            <Input
              autoComplete="username"
              prefix={<UserOutlined />}
              placeholder="在桌面端全局设置中配置"
              disabled={loadingStatus || authStatus?.configured === false}
            />
          </Form.Item>
          <Form.Item
            label="管理员密码"
            name="password"
            rules={[{ required: true, message: '请输入桌面端配置的管理员密码' }]}
          >
            <Input.Password
              autoComplete="current-password"
              prefix={<LockOutlined />}
              placeholder="请输入管理员密码"
              disabled={loadingStatus || authStatus?.configured === false}
            />
          </Form.Item>
          {captchaRequired && (
            <div className="web-login-captcha">
              <div className="web-login-captcha__head">
                <div>
                  <strong>安全验证码</strong>
                  <span>首次登录失败后一小时内，需要输入图中的字符串验证码。</span>
                </div>
                <Button
                  aria-label="刷新验证码"
                  icon={<ReloadOutlined />}
                  loading={loadingCaptcha}
                  onClick={() => void refreshCaptcha()}
                  size="small"
                >
                  换一张
                </Button>
              </div>
              <div className="web-login-captcha__body">
                <div className="web-login-captcha__image">
                  {captchaImageUrl ? (
                    <img src={captchaImageUrl} alt="字符串验证码" />
                  ) : (
                    <span>{loadingCaptcha ? '正在生成验证码...' : '请刷新验证码'}</span>
                  )}
                </div>
                <Form.Item
                  label="验证码"
                  name="captchaAnswer"
                  rules={[{ required: true, message: '请输入图中的字符串验证码' }]}
                >
                  <Input
                    autoComplete="one-time-code"
                    disabled={loadingStatus || loadingCaptcha || authStatus?.configured === false}
                    placeholder="输入验证码"
                  />
                </Form.Item>
              </div>
            </div>
          )}
          <Button
            block
            className="web-login-submit"
            htmlType="submit"
            icon={<SafetyCertificateOutlined />}
            loading={loadingStatus || loadingCaptcha || submitting}
            type="primary"
            disabled={authStatus?.configured === false}
          >
            登录 Web 控制台
          </Button>
        </Form>
      </section>
    </div>
  )
}
