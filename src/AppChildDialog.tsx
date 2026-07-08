import { Suspense, lazy } from 'react'
import { Modal } from 'antd'
import { PanelLoading } from './appShell'
import type { ServerInstance } from './types'

const AddInstanceWindow = lazy(() => import('./AddInstanceWindow'))
const RconWindow = lazy(() => import('./RconWindow'))
const SettingsWindow = lazy(() => import('./SettingsWindow'))

export type WebDialogState =
  | { type: 'settings' }
  | { type: 'add-instance'; params: URLSearchParams }
  | { type: 'rcon'; instance: ServerInstance }
  | null

interface AppChildDialogProps {
  dialog: WebDialogState
  onClose: () => void
}

function dialogWidth(dialog: WebDialogState) {
  if (dialog?.type === 'rcon') return 1080
  if (dialog?.type === 'settings') return 900
  return 780
}

export default function AppChildDialog({ dialog, onClose }: AppChildDialogProps) {
  return (
    <Modal
      className="web-child-dialog"
      footer={null}
      open={dialog !== null}
      onCancel={onClose}
      title={null}
      width={dialogWidth(dialog)}
    >
      <Suspense fallback={<PanelLoading text="正在加载弹窗..." />}>
        {dialog?.type === 'settings' ? (
          <SettingsWindow onClose={onClose} />
        ) : dialog?.type === 'add-instance' ? (
          <AddInstanceWindow
            initialParams={dialog.params}
            onClose={onClose}
          />
        ) : dialog?.type === 'rcon' ? (
          <RconWindow
            instanceId={dialog.instance.id}
            name={dialog.instance.name}
            rconPort={dialog.instance.rconPort}
          />
        ) : null}
      </Suspense>
    </Modal>
  )
}
