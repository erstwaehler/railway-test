/**
 * TanStack Query Hooks
 * 
 * Pre-configured query hooks for common API operations.
 * Uses TanStack Query for caching, refetching, and state management.
 */

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { eventsApi, participantsApi } from './api-client'
import type { CreateEvent, CreateParticipant, UpdateParticipantStatus } from './types'

// Query keys factory for type-safety and consistency
export const queryKeys = {
  events: {
    all: ['events'] as const,
    lists: () => [...queryKeys.events.all, 'list'] as const,
    list: (filters?: unknown) => [...queryKeys.events.lists(), filters] as const,
    details: () => [...queryKeys.events.all, 'detail'] as const,
    detail: (id: string) => [...queryKeys.events.details(), id] as const,
  },
  participants: {
    all: ['participants'] as const,
    lists: () => [...queryKeys.participants.all, 'list'] as const,
    byEvent: (eventId: string) => [...queryKeys.participants.lists(), eventId] as const,
    details: () => [...queryKeys.participants.all, 'detail'] as const,
    detail: (id: string) => [...queryKeys.participants.details(), id] as const,
  },
}

// ===== EVENT QUERIES =====

export function useEvents() {
  return useQuery({
    queryKey: queryKeys.events.lists(),
    queryFn: () => eventsApi.listEvents(),
  })
}

export function useEvent(id: string) {
  return useQuery({
    queryKey: queryKeys.events.detail(id),
    queryFn: () => eventsApi.getEvent(id),
    enabled: !!id,
  })
}

export function useCreateEvent() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: CreateEvent) => eventsApi.createEvent(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.events.lists() })
    },
  })
}

export function useUpdateEvent() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ id, data }: { id: string; data: CreateEvent }) =>
      eventsApi.updateEvent(id, data),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.events.lists() })
      queryClient.invalidateQueries({ queryKey: queryKeys.events.detail(variables.id) })
    },
  })
}

export function useDeleteEvent() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (id: string) => eventsApi.deleteEvent(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.events.lists() })
    },
  })
}

// ===== PARTICIPANT QUERIES =====

export function useParticipants(eventId: string) {
  return useQuery({
    queryKey: queryKeys.participants.byEvent(eventId),
    queryFn: () => participantsApi.listParticipants(eventId),
    enabled: !!eventId,
  })
}

export function useParticipant(id: string) {
  return useQuery({
    queryKey: queryKeys.participants.detail(id),
    queryFn: () => participantsApi.getParticipant(id),
    enabled: !!id,
  })
}

export function useCreateParticipant() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: CreateParticipant) => participantsApi.createParticipant(data),
    onSuccess: (data) => {
      queryClient.invalidateQueries({ 
        queryKey: queryKeys.participants.byEvent(data.event_id) 
      })
    },
  })
}

export function useUpdateParticipantStatus() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ id, data }: { id: string; data: UpdateParticipantStatus }) =>
      participantsApi.updateParticipantStatus(id, data),
    onSuccess: (data) => {
      queryClient.invalidateQueries({ 
        queryKey: queryKeys.participants.byEvent(data.event_id) 
      })
      queryClient.invalidateQueries({ 
        queryKey: queryKeys.participants.detail(data.id) 
      })
    },
  })
}

export function useDeleteParticipant() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (id: string) => participantsApi.deleteParticipant(id),
    onSuccess: () => {
      // Invalidate all participant lists since we don't know which event
      queryClient.invalidateQueries({ 
        queryKey: queryKeys.participants.lists() 
      })
    },
  })
}

