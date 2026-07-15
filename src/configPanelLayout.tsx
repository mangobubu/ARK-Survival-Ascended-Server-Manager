import {
  Children,
  cloneElement,
  createContext,
  isValidElement,
  type ReactElement,
  type ReactNode,
  useContext,
  useState,
} from 'react'
import { InfoCircleOutlined, RightOutlined } from '@ant-design/icons'
import { Input, InputNumber, Space, Tooltip } from 'antd'

const AccordionContext = createContext<{
  activeSection: string | null
  setActiveSection: (section: string | null) => void
  forceExpand: boolean
} | null>(null)

export function AccordionGroup({ children, className = '', forceExpand = false }: { children: ReactNode; className?: string; forceExpand?: boolean }) {
  const [activeSection, setActiveSection] = useState<string | null>(null)

  return (
    <AccordionContext.Provider value={{ activeSection, setActiveSection, forceExpand }}>
      <div className={`settings-accordion ${className}`}>{children}</div>
    </AccordionContext.Provider>
  )
}

export function SectionCard({ title, icon, note, children, className = '' }: { title: string; icon?: ReactNode; note?: string; children: ReactNode; className?: string }) {
  const accordion = useContext(AccordionContext)
  const expanded = accordion?.forceExpand || !accordion || accordion.activeSection === title

  return (
    <section className={`setting-card ${expanded ? 'setting-card--expanded' : ''} ${className}`}>
      <button
        type="button"
        className="setting-card__header"
        aria-expanded={expanded}
        onClick={() => {
          if (!accordion || accordion.forceExpand) return
          accordion.setActiveSection(expanded ? null : title)
        }}
      >
        <span className="setting-card__icon">{icon}</span>
        <span>{title}</span>
        {note && <span className="setting-card__note">{note}</span>}
        {accordion && <RightOutlined className="setting-card__chevron" />}
      </button>
      {expanded && <div className="setting-card__body">{children}</div>}
    </section>
  )
}

export function Field({ label, tip, children, wide = false }: { label: string; tip?: string; children: ReactNode; wide?: boolean }) {
  return (
    <div className={`config-field ${wide ? 'config-field--wide' : ''}`}>
      <div className="config-field__label">
        <span>{label}</span>
        {tip && <Tooltip title={tip}><InfoCircleOutlined /></Tooltip>}
      </div>
      <div className="config-field__control">{children}</div>
    </div>
  )
}

export function NumberField({
  value,
  onChange,
  min = 0,
  max,
  step = 1,
  addonAfter,
  disabled = false,
}: {
  value: number
  onChange: (value: number) => void
  min?: number
  max?: number
  step?: number
  addonAfter?: string
  disabled?: boolean
}) {
  if (addonAfter) {
    return (
      <Space.Compact>
        <InputNumber disabled={disabled} value={value} min={min} max={max} step={step} onChange={(next) => onChange(next ?? min)} />
        <Input value={addonAfter} disabled style={{ width: '48px', textAlign: 'center', padding: '4px 0' }} />
      </Space.Compact>
    )
  }
  return <InputNumber disabled={disabled} value={value} min={min} max={max} step={step} onChange={(next) => onChange(next ?? min)} />
}

type SearchableElementProps = {
  children?: ReactNode
  label?: ReactNode
  note?: ReactNode
  options?: unknown
  placeholder?: ReactNode
  tip?: ReactNode
  title?: ReactNode
}

const searchablePropNames = ['label', 'note', 'options', 'placeholder', 'tip', 'title'] as const

export function normalizeSearchText(value: unknown) {
  return String(value ?? '').trim().toLocaleLowerCase()
}

function textFromSearchValue(value: unknown): string {
  if (value == null || typeof value === 'boolean') return ''
  if (typeof value === 'string' || typeof value === 'number') return String(value)
  if (Array.isArray(value)) return value.map(textFromSearchValue).join(' ')
  if (isValidElement(value)) return collectSearchText(value)
  if (typeof value === 'object') {
    return Object.values(value as Record<string, unknown>).map(textFromSearchValue).join(' ')
  }
  return ''
}

function getSearchableProps(node: ReactNode) {
  return isValidElement(node) ? node.props as SearchableElementProps : null
}

function collectOwnSearchText(node: ReactNode) {
  if (typeof node === 'string' || typeof node === 'number') return String(node)
  const props = getSearchableProps(node)
  if (!props) return ''
  return searchablePropNames.map((name) => textFromSearchValue(props[name])).join(' ')
}

function collectSearchText(node: ReactNode): string {
  if (node == null || typeof node === 'boolean') return ''
  if (typeof node === 'string' || typeof node === 'number') return String(node)
  if (Array.isArray(node)) return node.map(collectSearchText).join(' ')

  const props = getSearchableProps(node)
  if (!props) return ''

  const childText = Children.toArray(props.children).map(collectSearchText).join(' ')
  return `${collectOwnSearchText(node)} ${childText}`
}

export function hasRenderableNode(node: ReactNode): boolean {
  if (node == null || typeof node === 'boolean') return false
  if (Array.isArray(node)) return node.some(hasRenderableNode)
  return true
}

export function filterSearchNode(node: ReactNode, query: string): ReactNode {
  if (!query) return node
  if (node == null || typeof node === 'boolean') return null
  if (typeof node === 'string' || typeof node === 'number') {
    return normalizeSearchText(node).includes(query) ? node : null
  }
  if (Array.isArray(node)) {
    const filtered = Children.toArray(node).map((child) => filterSearchNode(child, query)).filter(hasRenderableNode)
    return filtered.length > 0 ? filtered : null
  }
  if (!isValidElement(node)) return null

  const props = node.props as SearchableElementProps
  const ownMatches = normalizeSearchText(collectOwnSearchText(node)).includes(query)
  const wholeMatches = normalizeSearchText(collectSearchText(node)).includes(query)

  if (!wholeMatches) return null
  if (node.type === Field) return node
  if (node.type === SectionCard && ownMatches) return node

  const filteredChildren = filterSearchNode(props.children, query)
  if (hasRenderableNode(filteredChildren)) {
    return cloneElement(node as ReactElement<SearchableElementProps>, undefined, filteredChildren)
  }

  return ownMatches ? node : null
}
