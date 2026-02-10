import { QueryClient } from '@tanstack/react-query'
import { createRouter } from '@tanstack/react-router'
import { getRouterManifest } from '@tanstack/start/router-manifest'
import {
  createStartHandler,
  defaultStreamHandler,
} from '@tanstack/start/server'
import { routeTree } from './routeTree.gen'
import { ServerApp } from './ssr'

// Create a query client instance
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 60 * 1000, // 1 minute
      gcTime: 5 * 60 * 1000, // 5 minutes
      retry: 1,
      refetchOnWindowFocus: false,
    },
  },
})

// Export getRouter for client-side hydration
export function getRouter() {
  return createRouter({
    routeTree,
    context: {
      queryClient,
    },
    defaultPreload: 'intent',
  })
}

// Server handler export
export default createStartHandler({
  createRouter: ServerApp,
  getRouterManifest,
})(defaultStreamHandler)
