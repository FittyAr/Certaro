import type { TipoEvento } from '@/stores/useCalendarioStore'

export function getBadgeClass(tipo: TipoEvento, esVirtual: boolean): string {
  if (esVirtual) {
    return 'bg-warning/20 text-warning border-warning/30'
  }
  switch (tipo) {
    case 'Trabajo':
      return 'bg-primary/20 text-primary border-primary/30'
    case 'Reunion':
      return 'bg-info/20 text-info border-info/30'
    case 'Mantenimiento':
      return 'bg-warning/20 text-warning border-warning/30'
    case 'Entrega':
      return 'bg-success/20 text-success border-success/30'
    default:
      return 'bg-muted text-foreground border-border'
  }
}
