import { useEffect, useMemo, useState } from 'react'
import { DeleteOutlined, PlusOutlined, SearchOutlined } from '@ant-design/icons'
import { Button, Empty, Input, InputNumber, Modal, Select, Switch, Tag, Typography } from 'antd'
import { arkStackableItemOptions } from './arkStackableItemOptions'
import { arkStackableItemZhCategories, arkStackableItemZhLabels } from './arkStackableItemLocalizations'
import { normalizeSearchText } from './configPanelLayout'
import type { GlobalSettings, ItemStackOption, ItemStackOverride } from './types'

const { Paragraph } = Typography

type AppLanguage = GlobalSettings['language']

type ItemStackLanguageText = {
  defaultStackInOption: (defaultStackSize: number) => string
  defaultStackMeta: (defaultStackSize: number, category: string) => string
  previewPlaceholder: string
  unselectedItem: string
}

export const itemStackLanguageText: Record<AppLanguage, ItemStackLanguageText> = {
  'zh-CN': {
    defaultStackInOption: (defaultStackSize) => `默认 ${defaultStackSize}`,
    defaultStackMeta: (defaultStackSize, category) => `默认堆叠 ${defaultStackSize} · ${category}`,
    previewPlaceholder: '请选择物品后生成 ConfigOverrideItemMaxQuantity 预览',
    unselectedItem: '未选择物品',
  },
  'en-US': {
    defaultStackInOption: (defaultStackSize) => `Default ${defaultStackSize}`,
    defaultStackMeta: (defaultStackSize, category) => `Default stack ${defaultStackSize} · ${category}`,
    previewPlaceholder: 'Select an item to generate ConfigOverrideItemMaxQuantity preview',
    unselectedItem: 'No item selected',
  },
}

export function getItemStackItemLabel(item: ItemStackOption, language: AppLanguage) {
  return language === 'zh-CN' ? item.zhLabel ?? arkStackableItemZhLabels[item.classString] ?? item.label : item.label
}

export function getItemStackItemCategory(item: ItemStackOption, language: AppLanguage) {
  return language === 'zh-CN' ? item.zhCategory ?? arkStackableItemZhCategories[item.category] ?? item.category : item.category
}

function formatStackableItemOptionLabel(item: ItemStackOption, language: AppLanguage) {
  const displayLabel = getItemStackItemLabel(item, language)
  const defaultStackText = itemStackLanguageText[language].defaultStackInOption(item.defaultStackSize)
  return language === 'zh-CN' ? `${displayLabel}（${defaultStackText}）` : `${displayLabel} (${defaultStackText})`
}

function buildItemStackOverridePreview(override: ItemStackOverride, language: AppLanguage) {
  const itemClassString = override.itemClassString.trim()
  if (!itemClassString) return itemStackLanguageText[language].previewPlaceholder

  const maxItemQuantity = Math.max(1, Math.trunc(override.maxItemQuantity || 1))
  const ignoreMultiplier = override.ignoreMultiplier ? 'True' : 'False'
  return 'ConfigOverrideItemMaxQuantity=(ItemClassString="' + itemClassString + '",Quantity=(MaxItemQuantity=' + maxItemQuantity + ',bIgnoreMultiplier=' + ignoreMultiplier + '))'
}

interface ItemStackOverrideModalProps {
  language: AppLanguage
  open: boolean
  overrides: ItemStackOverride[]
  onChange: (next: ItemStackOverride[]) => void
  onClose: () => void
}

export function ItemStackOverrideModal({
  language,
  open,
  overrides,
  onChange,
  onClose,
}: ItemStackOverrideModalProps) {
  const [search, setSearch] = useState('')
  const itemStackText = itemStackLanguageText[language]

  useEffect(() => {
    if (open) setSearch('')
  }, [open])

  const stackableItemSelectOptions = useMemo(() => arkStackableItemOptions.map((item) => ({
    baseLabel: formatStackableItemOptionLabel(item, language),
    value: item.classString,
    searchText: [
      getItemStackItemLabel(item, language),
      item.label,
      item.classString,
      getItemStackItemCategory(item, language),
      item.category,
    ].join(' '),
  })), [language])

  const usedItemStackClassStrings = useMemo(() => new Set(
    overrides.map((override) => override.itemClassString.trim()).filter(Boolean)
  ), [overrides])

  const getStackableItemSelectOptions = (currentClassString: string) => stackableItemSelectOptions.map((option) => {
    const existsInOtherOverride = option.value !== currentClassString && usedItemStackClassStrings.has(option.value)
    return {
      ...option,
      label: existsInOtherOverride ? (
        <span className="item-stack-select-option item-stack-select-option--exists">
          <span>{option.baseLabel}</span>
          <Tag color="warning">已存在</Tag>
        </span>
      ) : option.baseLabel,
      disabled: existsInOtherOverride,
    }
  })

  const updateOverrides = (next: ItemStackOverride[]) => {
    onChange(next)
  }

  const updateOverride = (index: number, patch: Partial<ItemStackOverride>) => {
    updateOverrides(overrides.map((item, itemIndex) => (
      itemIndex === index ? { ...item, ...patch } : item
    )))
  }

  const addOverride = () => {
    updateOverrides([
      ...overrides,
      { itemClassString: '', maxItemQuantity: 100, ignoreMultiplier: true },
    ])
  }

  const removeOverride = (index: number) => {
    updateOverrides(overrides.filter((_, itemIndex) => itemIndex !== index))
  }

  const selectOverrideItem = (index: number, classString: string) => {
    if (overrides.some((item, itemIndex) => itemIndex !== index && item.itemClassString === classString)) return

    const selectedItem = arkStackableItemOptions.find((item) => item.classString === classString)
    updateOverride(index, {
      itemClassString: classString,
      maxItemQuantity: selectedItem?.defaultStackSize ?? overrides[index]?.maxItemQuantity ?? 100,
    })
  }

  const normalizedSearch = normalizeSearchText(search)
  const rows = overrides.map((override, index) => {
    const selectedItem = arkStackableItemOptions.find((item) => item.classString === override.itemClassString)
    const selectedItemLabel = selectedItem ? getItemStackItemLabel(selectedItem, language) : itemStackText.unselectedItem
    const selectedItemCategory = selectedItem ? getItemStackItemCategory(selectedItem, language) : ''
    const preview = buildItemStackOverridePreview(override, language)
    const searchText = normalizeSearchText([
      selectedItemLabel,
      selectedItem?.label,
      selectedItemCategory,
      selectedItem?.category,
      override.itemClassString,
      override.maxItemQuantity,
      preview,
    ].join(' '))

    return {
      index,
      override,
      selectedItem,
      selectedItemCategory,
      preview,
      hidden: normalizedSearch.length > 0 && !searchText.includes(normalizedSearch),
    }
  }).filter((row) => !row.hidden)

  return (
    <Modal
      title="物品单独叠加覆盖"
      open={open}
      onCancel={onClose}
      width={1120}
      centered
      maskClosable={false}
      className="item-stack-modal"
      footer={[
        <Button key="close" type="primary" onClick={onClose}>完成</Button>,
      ]}
    >
      <div className="item-stack-modal__intro">
        <Paragraph type="secondary">
          修改会实时写入当前实例配置；每次打开都会同步展示已有覆盖，保存配置后写入 Game.ini 的 ConfigOverrideItemMaxQuantity。
        </Paragraph>
      </div>
      <div className="item-stack-modal__toolbar">
        <Input
          allowClear
          aria-label="搜索已有物品覆盖"
          placeholder="搜索已有覆盖：物品名称、分类、ItemClassString 或数量"
          prefix={<SearchOutlined />}
          value={search}
          onChange={(event) => setSearch(event.target.value)}
        />
        <Button
          type="primary"
          icon={<PlusOutlined />}
          onClick={() => {
            setSearch('')
            addOverride()
          }}
        >
          添加物品覆盖
        </Button>
      </div>
      <div className="item-stack-modal__meta">
        <span>已显示 {rows.length} / {overrides.length} 条覆盖</span>
        <span>已有覆盖的物品会在选择列表中禁用，并标识“已存在”。</span>
      </div>
      {overrides.length === 0 ? (
        <Empty image={Empty.PRESENTED_IMAGE_SIMPLE} description="尚未添加物品覆盖">
          <Button type="primary" icon={<PlusOutlined />} onClick={addOverride}>立即添加</Button>
        </Empty>
      ) : rows.length === 0 ? (
        <Empty image={Empty.PRESENTED_IMAGE_SIMPLE} description="没有匹配的物品覆盖">
          <Button onClick={() => setSearch('')}>清空搜索</Button>
        </Empty>
      ) : (
        <div className="item-stack-editor-table-wrap">
          <table className="item-stack-editor-table">
            <thead>
              <tr>
                <th>物品</th>
                <th>最大叠加数量</th>
                <th>忽略全局倍率</th>
                <th>配置预览</th>
                <th>操作</th>
              </tr>
            </thead>
            <tbody>
              {rows.map((row) => (
                <tr key={row.index}>
                  <td className="item-stack-editor-table__item">
                    <Select
                      showSearch
                      className="item-stack-select"
                      value={row.override.itemClassString || undefined}
                      placeholder="搜索物品名称、分类或 ItemClassString"
                      options={getStackableItemSelectOptions(row.override.itemClassString)}
                      optionFilterProp="searchText"
                      popupMatchSelectWidth={false}
                      onChange={(value) => selectOverrideItem(row.index, value)}
                    />
                    {row.selectedItem && (
                      <div className="item-stack-row-meta">
                        {itemStackText.defaultStackMeta(row.selectedItem.defaultStackSize, row.selectedItemCategory)}
                      </div>
                    )}
                  </td>
                  <td>
                    <InputNumber
                      className="item-stack-quantity-input"
                      value={row.override.maxItemQuantity}
                      min={1}
                      max={1000000}
                      step={10}
                      onChange={(value) => {
                        const nextValue = Number(value ?? 1)
                        updateOverride(row.index, { maxItemQuantity: Math.max(1, Math.trunc(Number.isFinite(nextValue) ? nextValue : 1)) })
                      }}
                    />
                  </td>
                  <td>
                    <Switch checked={row.override.ignoreMultiplier} checkedChildren="是" unCheckedChildren="否" onChange={(value) => updateOverride(row.index, { ignoreMultiplier: value })} />
                  </td>
                  <td>
                    <div className="code-preview code-preview--single item-stack-preview">{row.preview}</div>
                  </td>
                  <td>
                    <Button danger size="small" icon={<DeleteOutlined />} onClick={() => removeOverride(row.index)}>
                      删除
                    </Button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </Modal>
  )
}
