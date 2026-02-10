/**
 * SSE Hook for real-time updates
 * 
 * Connects to SSE stream and invalidates relevant queries on events
 */

import { useEffect } from 'react'
import { useQueryClient } from '@tanstack/react-query'
import { SSEClient } from './sse-client'
import { queryKeys } from './queries'

const SSE_URL = import.meta.env.VITE_SSE_URL || 'http://localhost:3000/api/events/stream'

export function useEventSSE() {
  const queryClient = useQueryClient()

  useEffect(() => {
    // Create SSE client - use full URL since SSE_BASE_URL is already set
    const sseClient = new SSEClient('', {
      reconnect: true,
      reconnectInterval: 3000,
      maxRetries: 5,
    })

    // Override the URL to use the full stream endpoint
    const originalConnect = sseClient.connect.bind(sseClient)
    sseClient.connect = function() {
      // @ts-ignore - accessing private property
      this.url = SSE_URL
      originalConnect()
    }

    // Listen for all events and invalidate queries
    sseClient.on('message', (event) => {
      console.log('[SSE Hook] Received event:', event)

      // Invalidate all events and participants queries on any update
      queryClient.invalidateQueries({ queryKey: queryKeys.events.all })
      queryClient.invalidateQueries({ queryKey: queryKeys.participants.all })
    })

    // Connect to stream
    sseClient.connect()

    // Cleanup on unmount
    return () => {
      sseClient.disconnect()
    }
  }, [queryClient])
}
