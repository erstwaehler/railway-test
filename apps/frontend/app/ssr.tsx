import { QueryClient } from '@tanstack/react-query'
import { createRouter } from '@tanstack/react-router'
import { StartServer } from '@tanstack/start/server'
import { getRouterManifest } from '@tanstack/start/router-manifest'
import type { ReactNode } from 'react'
import { routeTree } from './routeTree.gen'

// Create a query client for SSR
export function createQueryClient() {
  return new QueryClient({
    defaultOptions: {
      queries: {
        staleTime: 60 * 1000,
        gcTime: 5 * 60 * 1000,
      },
    },
  })
}

export function ServerApp({ children }: { children?: ReactNode }) {
  const queryClient = createQueryClient()

  const router = createRouter({
    routeTree,
    context: {
      queryClient,
    },
    defaultPreload: 'intent',
  })

  return (
    <StartServer
      router={router}
      routerManifest={getRouterManifest()}
    >
      {children}
    </StartServer>
  )
}
