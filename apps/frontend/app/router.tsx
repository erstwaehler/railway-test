import { getRouterManifest } from '@tanstack/start/router-manifest'
import {
  createStartHandler,
  defaultStreamHandler,
} from '@tanstack/start/server'
import { ServerApp } from './ssr'

export default createStartHandler({
  createRouter: ServerApp,
  getRouterManifest,
})(defaultStreamHandler)
