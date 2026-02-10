import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { createRouter } from '@tanstack/react-router'
import { StartClient } from '@tanstack/start'
import { StrictMode } from 'react'
import { hydrateRoot } from 'react-dom/client'
import { routeTree } from './routeTree.gen'

// Create a query client instance
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 60 * 1000, // 1 minute
      gcTime: 5 * 60 * 1000, // 5 minutes (formerly cacheTime)
      retry: 1,
      refetchOnWindowFocus: false,
    },
  },
})

// Create a router instance
const router = createRouter({
  routeTree,
  context: {
    queryClient,
  },
  defaultPreload: 'intent',
})

// Type-safe router declaration
declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router
  }
}

hydrateRoot(
  document.getElementById('root')!,
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      <StartClient router={router} />
    </QueryClientProvider>
  </StrictMode>
)
