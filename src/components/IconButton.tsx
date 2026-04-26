import { Icon, type IconName } from './Icon.tsx'

type IconButtonProps = {
  active?: boolean
  badge?: number | string
  className?: string
  disabled?: boolean
  icon: IconName
  label: string
  onClick?: () => void
  type?: 'button' | 'submit'
}

function IconButton({
  active = false,
  badge,
  className,
  disabled,
  icon,
  label,
  onClick,
  type = 'button',
}: IconButtonProps) {
  const classes = [
    'icon-button',
    active ? 'is-active' : '',
    className ?? '',
  ].filter(Boolean).join(' ')

  return (
    <button
      aria-label={label}
      className={classes}
      disabled={disabled}
      onClick={onClick}
      title={label}
      type={type}
    >
      <Icon name={icon} />
      {badge ? <span className="icon-button__badge">{badge}</span> : null}
    </button>
  )
}

export { IconButton }
