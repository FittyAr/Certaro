export interface HelpTopic {
  id: string
  title: string
  subtitle: string
  purpose: string
  workflow: string[]
  strengths: string[]
  limitations: string[]
  tips: string[]
}

import { kanbanHelp } from './help/kanban'
import { dashboardHelp } from './help/dashboard'
import { finanzasHelp } from './help/finanzas'
import { obrasHelp } from './help/obras'
import { personalHelp } from './help/personal'
import { sistemaHelp } from './help/sistema'

export const HELP_REGISTRY: Record<string, HelpTopic> = {
  ...kanbanHelp,
  ...dashboardHelp,
  ...finanzasHelp,
  ...obrasHelp,
  ...personalHelp,
  ...sistemaHelp,
}
