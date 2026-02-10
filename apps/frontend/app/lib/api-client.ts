/**
 * API Client for backend communication
 * 
 * Base configuration for making HTTP requests to the backend API.
 * Adjust the baseURL based on your environment configuration.
 */

const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000/api'

export interface ApiError {
  message: string
  status: number
}

export class ApiClient {
  private baseURL: string

  constructor(baseURL: string = API_BASE_URL) {
    this.baseURL = baseURL
  }

  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const url = `${this.baseURL}${endpoint}`
    
    const response = await fetch(url, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        ...options.headers,
      },
    })

    if (!response.ok) {
      const error: ApiError = {
        message: `API request failed: ${response.statusText}`,
        status: response.status,
      }
      throw error
    }

    return response.json()
  }

  async get<T>(endpoint: string, options?: RequestInit): Promise<T> {
    return this.request<T>(endpoint, { ...options, method: 'GET' })
  }

  async post<T>(
    endpoint: string,
    data?: unknown,
    options?: RequestInit
  ): Promise<T> {
    return this.request<T>(endpoint, {
      ...options,
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  async put<T>(
    endpoint: string,
    data?: unknown,
    options?: RequestInit
  ): Promise<T> {
    return this.request<T>(endpoint, {
      ...options,
      method: 'PUT',
      body: JSON.stringify(data),
    })
  }

  async delete<T>(endpoint: string, options?: RequestInit): Promise<T> {
    const url = `${this.baseURL}${endpoint}`
    
    const response = await fetch(url, {
      ...options,
      method: 'DELETE',
      headers: {
        'Content-Type': 'application/json',
        ...options?.headers,
      },
    })

    if (!response.ok) {
      const error: ApiError = {
        message: `API request failed: ${response.statusText}`,
        status: response.status,
      }
      throw error
    }

    // Handle 204 No Content response
    if (response.status === 204) {
      return undefined as T
    }

    return response.json()
  }
}

// Export singleton instance
export const apiClient = new ApiClient()

// Typed API methods
import type { Event, CreateEvent, Participant, CreateParticipant, UpdateParticipantStatus } from './types'

export const eventsApi = {
  listEvents: () => apiClient.get<Event[]>('/events'),
  getEvent: (id: string) => apiClient.get<Event>(`/events/${id}`),
  createEvent: (data: CreateEvent) => apiClient.post<Event>('/events', data),
  updateEvent: (id: string, data: CreateEvent) => apiClient.put<Event>(`/events/${id}`, data),
  deleteEvent: (id: string) => apiClient.delete<void>(`/events/${id}`),
}

export const participantsApi = {
  listParticipants: (eventId: string) => apiClient.get<Participant[]>(`/events/${eventId}/participants`),
  getParticipant: (id: string) => apiClient.get<Participant>(`/participants/${id}`),
  createParticipant: (data: CreateParticipant) => apiClient.post<Participant>('/participants', data),
  updateParticipantStatus: (id: string, data: UpdateParticipantStatus) => apiClient.put<Participant>(`/participants/${id}`, data),
  deleteParticipant: (id: string) => apiClient.delete<void>(`/participants/${id}`),
}
