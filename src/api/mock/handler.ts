import { handleSystemCommand } from './handlers/system'
import { handleMovimientosCommand } from './handlers/movimientos'
import { handleComercialCommand } from './handlers/comercial'
import { handleProyectosCommand } from './handlers/proyectos'
import { handlePersonalCommand } from './handlers/personal'
import { handleCatalogsCommand } from './handlers/catalogs'

export function mockBrowserCommand<T>(command: string, args?: Record<string, unknown>): T {
  const sys = handleSystemCommand<T>(command, args)
  if (sys !== undefined) return sys

  const mov = handleMovimientosCommand<T>(command, args)
  if (mov !== undefined) return mov

  const com = handleComercialCommand<T>(command, args)
  if (com !== undefined) return com

  const pro = handleProyectosCommand<T>(command, args)
  if (pro !== undefined) return pro

  const per = handlePersonalCommand<T>(command, args)
  if (per !== undefined) return per

  const cat = handleCatalogsCommand<T>(command, args)
  if (cat !== undefined) return cat

  return null as unknown as T
}
