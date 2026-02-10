/**
 * SSE Hook for real-time updates
 * 
 * Connects to SSE stream and invalidates relevant queries on events
 */

import { useEffect } from 'react'
import { useQueryClient } from '@tanstack/react-query'
import { SSEClient } from './sse-client'
import { queryKeys } from './queries'

export function useEventSSE() {
  const queryClient = useQueryClient()

  useEffect(() => {
    // Create SSE client
    const sseClient = new SSEClient('', {
      reconnect: true,
      reconnectInterval: 3000,
      maxRetries: 5,
    })

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
