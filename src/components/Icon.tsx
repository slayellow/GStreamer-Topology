type IconName =
  | 'arrowLeft'
  | 'close'
  | 'diagnostics'
  | 'fileText'
  | 'folderOpen'
  | 'info'
  | 'monitor'
  | 'panelRight'
  | 'server'

type IconProps = {
  name: IconName
}

function Icon({ name }: IconProps) {
  const commonProps = {
    'aria-hidden': true,
    className: 'ui-icon',
    fill: 'none',
    focusable: false,
    stroke: 'currentColor',
    strokeLinecap: 'round' as const,
    strokeLinejoin: 'round' as const,
    strokeWidth: 2,
    viewBox: '0 0 24 24',
  }

  switch (name) {
    case 'arrowLeft':
      return (
        <svg {...commonProps}>
          <path d="M19 12H5" />
          <path d="m12 19-7-7 7-7" />
        </svg>
      )
    case 'close':
      return (
        <svg {...commonProps}>
          <path d="M18 6 6 18" />
          <path d="m6 6 12 12" />
        </svg>
      )
    case 'diagnostics':
      return (
        <svg {...commonProps}>
          <path d="m12 3 9 16H3L12 3Z" />
          <path d="M12 9v4" />
          <path d="M12 17h.01" />
        </svg>
      )
    case 'fileText':
      return (
        <svg {...commonProps}>
          <path d="M14 3H6a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9Z" />
          <path d="M14 3v6h6" />
          <path d="M8 13h8" />
          <path d="M8 17h5" />
        </svg>
      )
    case 'folderOpen':
      return (
        <svg {...commonProps}>
          <path d="M3 7a2 2 0 0 1 2-2h5l2 2h7a2 2 0 0 1 2 2v2" />
          <path d="M3 11h18l-2 8a2 2 0 0 1-2 1H5a2 2 0 0 1-2-1Z" />
        </svg>
      )
    case 'info':
      return (
        <svg {...commonProps}>
          <circle cx="12" cy="12" r="9" />
          <path d="M12 11v5" />
          <path d="M12 8h.01" />
        </svg>
      )
    case 'monitor':
      return (
        <svg {...commonProps}>
          <rect height="12" rx="2" width="18" x="3" y="4" />
          <path d="M8 20h8" />
          <path d="M12 16v4" />
        </svg>
      )
    case 'panelRight':
      return (
        <svg {...commonProps}>
          <rect height="16" rx="2" width="18" x="3" y="4" />
          <path d="M15 4v16" />
          <path d="M7 8h4" />
          <path d="M7 12h4" />
        </svg>
      )
    case 'server':
      return (
        <svg {...commonProps}>
          <rect height="7" rx="2" width="18" x="3" y="4" />
          <rect height="7" rx="2" width="18" x="3" y="13" />
          <path d="M7 8h.01" />
          <path d="M7 17h.01" />
        </svg>
      )
  }
}

export { Icon }
export type { IconName }
