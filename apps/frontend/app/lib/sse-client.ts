/**
 * SSE (Server-Sent Events) Client
 * 
 * Handles real-time event streaming from the backend.
 * Provides type-safe event handling and automatic reconnection.
 */

const SSE_BASE_URL = import.meta.env.VITE_SSE_URL || 'http://localhost:3000/api/events/stream'

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
  private nativeHandlers = new Map<string, (event: MessageEvent) => void>()

  constructor(endpoint: string, options: SSEClientOptions = {}) {
    this.url = endpoint ? `${SSE_BASE_URL}${endpoint}` : SSE_BASE_URL
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

      this.attachCustomListeners()
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
      this.addNativeListener(eventType)
    }
  }

  off(eventType: string, callback: (event: SSEEvent) => void): void {
    const listeners = this.listeners.get(eventType)
    if (listeners) {
      listeners.delete(callback)
    }

    if (listeners && listeners.size === 0 && this.eventSource && eventType !== 'message') {
      const handler = this.nativeHandlers.get(eventType)
      if (handler) {
        this.eventSource.removeEventListener(eventType, handler)
        this.nativeHandlers.delete(eventType)
      }
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
      this.nativeHandlers.clear()
      console.log('[SSE] Disconnected')
    }
  }

  isConnected(): boolean {
    return this.eventSource !== null && this.eventSource.readyState === EventSource.OPEN
  }

  private addNativeListener(eventType: string): void {
    if (!this.eventSource || this.nativeHandlers.has(eventType)) {
      return
    }
    const handler = (e: MessageEvent) => {
      this.handleMessage(e)
    }
    this.eventSource.addEventListener(eventType, handler)
    this.nativeHandlers.set(eventType, handler)
  }

  private attachCustomListeners(): void {
    if (!this.eventSource) {
      return
    }
    this.listeners.forEach((_listeners, eventType) => {
      if (eventType !== 'message') {
        this.addNativeListener(eventType)
      }
    })
  }
}

// Example usage helper
export function createEventSSEClient() {
  return new SSEClient('')
}
