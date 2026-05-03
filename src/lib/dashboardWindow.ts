export const DASHBOARD_WIDGET_WINDOW = {
  width: 960,
  height: 720,
  minWidth: 960,
  minHeight: 720,
  maxWidth: 960,
  maxHeight: 720,
  resizable: false,
  maximizable: false,
} as const

const MODAL_VIEWPORT_PADDING = 32

export function getDashboardShellStyle() {
  return {
    maxWidth: `${DASHBOARD_WIDGET_WINDOW.width}px`,
    maxHeight: `${DASHBOARD_WIDGET_WINDOW.height}px`,
  } as const
}

export function getDashboardModalStyle() {
  return {
    maxHeight: `min(${DASHBOARD_WIDGET_WINDOW.height - MODAL_VIEWPORT_PADDING}px, calc(100vh - ${MODAL_VIEWPORT_PADDING}px))`,
  } as const
}
