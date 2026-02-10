/**
 * SSE (Server-Sent Events) Client
 * 
 * Handles real-time event streaming from the backend.
 * Provides type-safe event handling and automatic reconnection.
 */

const SSE_BASE_URL = import.meta.env.VITE_SSE_URL || 'http://localhost:3000/sse'

export interface SSEEvent<T = unknown> {
  type: string
  data: T
  id?: string
  retry?: number
}

export interface SSEClientOptions {
  reconnect?: boolean
  reconnectInterval?: number
  maxRetries?: number
}

export class SSEClient {
  private eventSource: EventSource | null = null
  private url: string
  private options: SSEClientOptions
  private retryCount = 0
  private listeners = new Map<string, Set<(event: SSEEvent) => void>>()

  constructor(endpoint: string, options: SSEClientOptions = {}) {
    this.url = `${SSE_BASE_URL}${endpoint}`
    this.options = {
      reconnect: true,
      reconnectInterval: 3000,
      maxRetries: 5,
      ...options,
    }
  }

  connect(): void {
    if (this.eventSource) {
      return // Already connected
    }

    try {
      this.eventSource = new EventSource(this.url)

      this.eventSource.onopen = () => {
        console.log('[SSE] Connected to', this.url)
        this.retryCount = 0
      }

      this.eventSource.onerror = (error) => {
        console.error('[SSE] Connection error:', error)
        this.handleError()
      }

      this.eventSource.onmessage = (event) => {
        this.handleMessage(event)
      }
    } catch (error) {
      console.error('[SSE] Failed to connect:', error)
      this.handleError()
    }
  }

  on(eventType: string, callback: (event: SSEEvent) => void): void {
    if (!this.listeners.has(eventType)) {
      this.listeners.set(eventType, new Set())
    }
    this.listeners.get(eventType)!.add(callback)

    // Add native EventSource listener for custom event types
    if (this.eventSource && eventType !== 'message') {
      this.eventSource.addEventListener(eventType, (e) => {
        this.handleMessage(e as MessageEvent)
      })
    }
  }

  off(eventType: string, callback: (event: SSEEvent) => void): void {
    const listeners = this.listeners.get(eventType)
    if (listeners) {
      listeners.delete(callback)
    }
  }

  private handleMessage(event: MessageEvent): void {
    try {
      const data = JSON.parse(event.data)
      const sseEvent: SSEEvent = {
        type: event.type || 'message',
        data,
        id: event.lastEventId,
      }

      // Notify all listeners for this event type
      const listeners = this.listeners.get(sseEvent.type)
      if (listeners) {
        listeners.forEach((callback) => {
          callback(sseEvent)
        })
      }

      // Also notify 'message' listeners for all events
      if (sseEvent.type !== 'message') {
        const messageListeners = this.listeners.get('message')
        if (messageListeners) {
          messageListeners.forEach((callback) => {
            callback(sseEvent)
          })
        }
      }
    } catch (error) {
      console.error('[SSE] Failed to parse event data:', error)
    }
  }

  private handleError(): void {
    this.disconnect()

    if (
      this.options.reconnect &&
      this.retryCount < (this.options.maxRetries || 5)
    ) {
      this.retryCount++
      console.log(
        `[SSE] Reconnecting (${this.retryCount}/${this.options.maxRetries})...`
      )
      setTimeout(() => {
        this.connect()
      }, this.options.reconnectInterval)
    } else {
      console.error('[SSE] Max retries reached, giving up')
    }
  }

  disconnect(): void {
    if (this.eventSource) {
      this.eventSource.close()
      this.eventSource = null
      console.log('[SSE] Disconnected')
    }
  }

  isConnected(): boolean {
    return this.eventSource !== null && this.eventSource.readyState === EventSource.OPEN
  }
}

// Example usage helper
export function createEventSSEClient(eventId: string) {
  return new SSEClient(`/events/${eventId}`)
}
