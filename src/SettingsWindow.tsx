import { useEffect, useState } from 'react'
import { Form, Input, Radio, Switch, InputNumber, Button, Select, Space, Divider, Typography, message, Layout } from 'antd'
import { FolderOpenOutlined, SaveOutlined } from '@ant-design/icons'
import { getCurrentWindow } from '@tauri-apps/api/window'
import type { GlobalSettings } from './types'
import { defaultGlobalSettings } from './data'

const { Text, Title } = Typography
const { Header, Content, Footer } = Layout

export default function SettingsWindow() {
  const [form] = Form.useForm<GlobalSettings>()
  const [messageApi, contextHolder] = message.useMessage()
  const [settings, setSettings] = useState<GlobalSettings>(() => {
    try {
      return { ...defaultGlobalSettings, ...JSON.parse(localStorage.getItem('asa-global-settings') ?? '{}') }
    } catch {
      return defaultGlobalSettings
    }
  })

  useEffect(() => {
    form.setFieldsValue(settings)
  }, [settings, form])

  const handleFinish = (values: GlobalSettings) => {
    localStorage.setItem('asa-global-settings', JSON.stringify(values))
    messageApi.success('配置已保存')
    
    // Optional: send an event to main window to reload settings if needed
    // Then close the settings window
    setTimeout(() => {
      getCurrentWindow().close()
    }, 600)
  }

  const loginMode = Form.useWatch('steamCmdLoginMode', form)

  return (
    <Layout style={{ minHeight: '100vh', background: '#020b14' }}>
      {contextHolder}
      <Header style={{ background: '#061523', borderBottom: '1px solid #14334d', padding: '0 24px', display: 'flex', alignItems: 'center' }}>
        <Title level={4} style={{ margin: 0, color: '#dbe8f4' }}>全局设置</Title>
      </Header>
      
      <Content style={{ padding: '24px', overflowY: 'auto', height: 'calc(100vh - 130px)' }}>
        <Form
          form={form}
          layout="vertical"
          onFinish={handleFinish}
          initialValues={settings}
        >
          <Divider titlePlacement="left" style={{ marginTop: 0 }}>基础与界面 (General)</Divider>
          <Form.Item label="应用语言 (Language)" name="language">
            <Select>
              <Select.Option value="zh-CN">简体中文 (zh-CN)</Select.Option>
              <Select.Option value="en-US">English (en-US)</Select.Option>
            </Select>
          </Form.Item>
          <Form.Item label="应用主题 (Theme)" name="theme">
            <Radio.Group>
              <Radio.Button value="dark">暗色 (Dark)</Radio.Button>
              <Radio.Button value="light">亮色 (Light)</Radio.Button>
              <Radio.Button value="system">跟随系统 (System)</Radio.Button>
            </Radio.Group>
          </Form.Item>

          <Divider titlePlacement="left">SteamCMD 设置</Divider>
          <Form.Item label="SteamCMD 路径" name="steamCmdPath" tooltip="SteamCMD 可执行文件所在的目录">
            <Input 
              addonAfter={<FolderOpenOutlined style={{ cursor: 'pointer' }} />} 
              placeholder="例如：C:\SteamCMD" 
            />
          </Form.Item>
          <Form.Item label="登录模式" name="steamCmdLoginMode">
            <Radio.Group>
              <Radio.Button value="anonymous">匿名登录 (推荐)</Radio.Button>
              <Radio.Button value="account">Steam 账户登录</Radio.Button>
            </Radio.Group>
          </Form.Item>
          
          {loginMode === 'account' && (
            <div className="login-account-box" style={{ background: 'rgba(0,0,0,0.2)', padding: '12px 16px', borderRadius: 8, marginBottom: 24 }}>
              <Text type="warning" style={{ display: 'block', marginBottom: 12 }}>
                注意：ARK 服务端本身支持匿名下载，仅当您需要下载其他付费内容时才建议使用账户登录。
              </Text>
              <Form.Item label="Steam 账号" name="steamCmdUsername" style={{ marginBottom: 12 }}>
                <Input placeholder="输入您的 Steam 账号" />
              </Form.Item>
              <Form.Item label="Steam 密码" name="steamCmdPassword" style={{ marginBottom: 0 }}>
                <Input.Password placeholder="输入您的 Steam 密码" />
              </Form.Item>
            </div>
          )}

          <Divider titlePlacement="left">存储与路径设置</Divider>
          <Form.Item label="服务器总存储路径" name="serverStoragePath" tooltip="所有服务器实例将被默认安装到此目录下">
            <Input 
              addonAfter={<FolderOpenOutlined style={{ cursor: 'pointer' }} />} 
              placeholder="例如：D:\ASA-Servers" 
            />
          </Form.Item>
          <Form.Item label="备份存储路径" name="backupStoragePath" tooltip="自动备份文件的存放位置">
            <Input 
              addonAfter={<FolderOpenOutlined style={{ cursor: 'pointer' }} />} 
              placeholder="例如：D:\ASA-Backups" 
            />
          </Form.Item>

          <Divider titlePlacement="left">自动化与维护 (Automation)</Divider>
          <Form.Item label="启动时自动检查更新" name="autoUpdateOnStart" valuePropName="checked">
            <Switch checkedChildren="开启" unCheckedChildren="关闭" />
          </Form.Item>
          <Form.Item label="崩溃时自动重启服务端" name="autoRestartOnCrash" valuePropName="checked">
            <Switch checkedChildren="开启" unCheckedChildren="关闭" />
          </Form.Item>
          <Form.Item label="自动备份保留数量" name="maxBackupRetention" tooltip="自动备份最多保留的文件个数">
            <InputNumber min={1} max={100} style={{ width: '100%' }} addonAfter="个备份" />
          </Form.Item>

        </Form>
      </Content>
      <Footer style={{ background: '#061523', borderTop: '1px solid #14334d', padding: '16px 24px', textAlign: 'right' }}>
        <Space>
          <Button onClick={() => getCurrentWindow().close()}>取消</Button>
          <Button type="primary" icon={<SaveOutlined />} onClick={() => form.submit()}>
            保存配置
          </Button>
        </Space>
      </Footer>
    </Layout>
  )
}
