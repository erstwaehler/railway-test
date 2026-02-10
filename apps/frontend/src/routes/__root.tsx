import {
  Outlet,
  ScrollRestoration,
  createRootRoute,
} from '@tanstack/react-router'
import type { ReactNode } from 'react'
import { useEventSSE } from '../lib/use-event-sse'

export const Route = createRootRoute({
  component: RootComponent,
})

function RootComponent() {
  return (
    <RootDocument>
      <SSEProvider>
        <Outlet />
      </SSEProvider>
    </RootDocument>
  )
}

function SSEProvider({ children }: { children: ReactNode }) {
  // Connect to SSE stream for real-time updates
  useEventSSE()
  return <>{children}</>
}

function RootDocument({ children }: { children: ReactNode }) {
  return (
    <html lang="en">
      <head>
        <meta charSet="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <title>Railway Test - Events</title>
        <style>{`
          * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
          }
          body {
            font-family: system-ui, -apple-system, sans-serif;
            line-height: 1.5;
            color: #333;
          }
          a {
            color: #0070f3;
            text-decoration: none;
          }
          a:hover {
            text-decoration: underline;
          }
          button {
            font-family: inherit;
            cursor: pointer;
          }
          input, textarea {
            font-family: inherit;
          }
        `}</style>
      </head>
      <body>
        <div id="root">{children}</div>
        <ScrollRestoration />
      </body>
    </html>
  )
}
