/**
 * TanStack Query Hooks
 * 
 * Pre-configured query hooks for common API operations.
 * Uses TanStack Query for caching, refetching, and state management.
 */

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { eventsApi } from './api-client'

// Query keys factory for type-safety and consistency
export const queryKeys = {
  events: {
    all: ['events'] as const,
    lists: () => [...queryKeys.events.all, 'list'] as const,
    list: (filters?: unknown) => [...queryKeys.events.lists(), filters] as const,
    details: () => [...queryKeys.events.all, 'detail'] as const,
    detail: (id: string) => [...queryKeys.events.details(), id] as const,
  },
}

// Example: Fetch single event
export function useEvent(id: string) {
  return useQuery({
    queryKey: queryKeys.events.detail(id),
    queryFn: () => eventsApi.getEvent(id),
    enabled: !!id,
  })
}

// Example: Fetch all events
export function useEvents() {
  return useQuery({
    queryKey: queryKeys.events.lists(),
    queryFn: () => eventsApi.listEvents(),
  })
}

// Example: Create event mutation
export function useCreateEvent() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: unknown) => eventsApi.createEvent(data),
    onSuccess: () => {
      // Invalidate and refetch events list
      queryClient.invalidateQueries({ queryKey: queryKeys.events.lists() })
    },
  })
}
