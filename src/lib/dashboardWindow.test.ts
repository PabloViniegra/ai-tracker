import { describe, expect, it } from 'vitest'
import {
  DASHBOARD_WIDGET_WINDOW,
  getDashboardModalStyle,
  getDashboardShellStyle,
} from './dashboardWindow'

describe('dashboardWindow', () => {
  it('defines a fixed widget window configuration', () => {
    expect(DASHBOARD_WIDGET_WINDOW).toMatchObject({
      width: 960,
      height: 720,
      minWidth: 960,
      minHeight: 720,
      maxWidth: 960,
      maxHeight: 720,
      resizable: false,
      maximizable: false,
    })
  })

  it('returns shell style capped to the widget footprint', () => {
    expect(getDashboardShellStyle()).toEqual({
      maxWidth: '960px',
      maxHeight: '720px',
    })
  })

  it('caps modal height inside the fixed dashboard viewport', () => {
    expect(getDashboardModalStyle()).toEqual({
      maxHeight: 'min(688px, calc(100vh - 32px))',
    })
  })
})
