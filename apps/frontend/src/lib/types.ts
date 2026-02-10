/**
 * TypeScript types matching backend models
 */

export interface Event {
  id: string
  title: string
  description: string | null
  start_time: string
  end_time: string
  location: string | null
  max_participants: number | null
  created_at: string
  updated_at: string
}

export interface CreateEvent {
  title: string
  description?: string | null
  start_time: string
  end_time: string
  location?: string | null
  max_participants?: number | null
}

export type ParticipantStatus = 'registered' | 'confirmed' | 'cancelled' | 'waitlisted'

export interface Participant {
  id: string
  event_id: string
  name: string
  email: string
  status: ParticipantStatus
  registered_at: string
  updated_at: string
}

export interface CreateParticipant {
  event_id: string
  name: string
  email: string
}

export interface UpdateParticipantStatus {
  status: ParticipantStatus
}
