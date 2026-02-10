import { createFileRoute, Link } from '@tanstack/react-router'
import { useState } from 'react'
import { useEvent, useParticipants, useCreateParticipant, useDeleteParticipant } from '../../lib/queries'
import type { CreateParticipant } from '../../lib/types'

export const Route = createFileRoute('/events/$id')({
  component: EventDetailComponent,
})

function EventDetailComponent() {
  const { id } = Route.useParams()
  const { data: event, isLoading: eventLoading, error: eventError } = useEvent(id)
  const { data: participants, isLoading: participantsLoading } = useParticipants(id)
  const [showForm, setShowForm] = useState(false)

  if (eventLoading) {
    return (
      <div style={{ maxWidth: '1200px', margin: '0 auto', padding: '2rem' }}>
        <p>Loading event...</p>
      </div>
    )
  }

  if (eventError || !event) {
    return (
      <div style={{ maxWidth: '1200px', margin: '0 auto', padding: '2rem' }}>
        <div style={{
          padding: '1rem',
          backgroundColor: '#fee',
          border: '1px solid #fcc',
          borderRadius: '6px',
          color: '#c00',
          marginBottom: '1rem',
        }}>
          Error loading event: {eventError?.message || 'Event not found'}
        </div>
        <Link to="/">← Back to Events</Link>
      </div>
    )
  }

  return (
    <div style={{ maxWidth: '1200px', margin: '0 auto', padding: '2rem' }}>
      <Link to="/" style={{ display: 'inline-block', marginBottom: '1rem' }}>
        ← Back to Events
      </Link>

      <div style={{
        padding: '2rem',
        border: '1px solid #ddd',
        borderRadius: '8px',
        backgroundColor: 'white',
        marginBottom: '2rem',
      }}>
        <h1 style={{ fontSize: '2rem', marginBottom: '1rem' }}>{event.title}</h1>
        
        {event.description && (
          <p style={{ color: '#666', marginBottom: '1.5rem', fontSize: '1.125rem' }}>
            {event.description}
          </p>
        )}

        <div style={{ display: 'grid', gap: '0.75rem', fontSize: '1rem' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
            <strong>📅 Start:</strong>
            <span>{new Date(event.start_time).toLocaleString()}</span>
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
            <strong>⏰ End:</strong>
            <span>{new Date(event.end_time).toLocaleString()}</span>
          </div>
          {event.location && (
            <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
              <strong>📍 Location:</strong>
              <span>{event.location}</span>
            </div>
          )}
          {event.max_participants && (
            <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
              <strong>👥 Capacity:</strong>
              <span>
                {participants?.length || 0} / {event.max_participants} participants
              </span>
            </div>
          )}
        </div>
      </div>

      <div>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1rem' }}>
          <h2 style={{ fontSize: '1.5rem' }}>Participants</h2>
          <button
            type="button"
            onClick={() => setShowForm(!showForm)}
            style={{
              padding: '0.5rem 1rem',
              backgroundColor: '#0070f3',
              color: 'white',
              border: 'none',
              borderRadius: '6px',
              fontSize: '0.875rem',
              fontWeight: '500',
            }}
          >
            {showForm ? 'Cancel' : '+ Add Participant'}
          </button>
        </div>

        {showForm && (
          <AddParticipantForm eventId={id} onSuccess={() => setShowForm(false)} />
        )}

        {participantsLoading && <p>Loading participants...</p>}

        {participants && participants.length === 0 && (
          <p style={{ color: '#666', fontStyle: 'italic', padding: '2rem', textAlign: 'center' }}>
            No participants yet. Be the first to register!
          </p>
        )}

        {participants && participants.length > 0 && (
          <div style={{ display: 'grid', gap: '0.5rem' }}>
            {participants.map((participant) => (
              <ParticipantItem key={participant.id} participant={participant} />
            ))}
          </div>
        )}
      </div>
    </div>
  )
}

function AddParticipantForm({ eventId, onSuccess }: { eventId: string; onSuccess: () => void }) {
  const createParticipant = useCreateParticipant()
  const [formData, setFormData] = useState<Omit<CreateParticipant, 'event_id'>>({
    name: '',
    email: '',
  })

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    
    try {
      await createParticipant.mutateAsync({
        event_id: eventId,
        name: formData.name,
        email: formData.email,
      })
      setFormData({ name: '', email: '' })
      onSuccess()
    } catch (error) {
      console.error('Failed to add participant:', error)
    }
  }

  return (
    <form
      onSubmit={handleSubmit}
      style={{
        padding: '1.5rem',
        border: '1px solid #ddd',
        borderRadius: '8px',
        backgroundColor: '#f9f9f9',
        marginBottom: '1rem',
      }}
    >
      <h3 style={{ marginBottom: '1rem', fontSize: '1.125rem' }}>Add Participant</h3>
      
      <div style={{ display: 'grid', gap: '1rem' }}>
        <div>
          <label htmlFor="participant-name" style={{ display: 'block', marginBottom: '0.25rem', fontWeight: '500' }}>
            Name *
          </label>
          <input
            id="participant-name"
            type="text"
            required
            value={formData.name}
            onChange={(e) => setFormData({ ...formData, name: e.target.value })}
            style={{
              width: '100%',
              padding: '0.5rem',
              border: '1px solid #ddd',
              borderRadius: '4px',
              fontSize: '1rem',
            }}
          />
        </div>

        <div>
          <label htmlFor="participant-email" style={{ display: 'block', marginBottom: '0.25rem', fontWeight: '500' }}>
            Email *
          </label>
          <input
            id="participant-email"
            type="email"
            required
            value={formData.email}
            onChange={(e) => setFormData({ ...formData, email: e.target.value })}
            style={{
              width: '100%',
              padding: '0.5rem',
              border: '1px solid #ddd',
              borderRadius: '4px',
              fontSize: '1rem',
            }}
          />
        </div>

        <div style={{ display: 'flex', gap: '1rem', marginTop: '0.5rem' }}>
          <button
            type="submit"
            disabled={createParticipant.isPending}
            style={{
              padding: '0.5rem 1rem',
              backgroundColor: '#0070f3',
              color: 'white',
              border: 'none',
              borderRadius: '6px',
              fontSize: '1rem',
              fontWeight: '500',
              opacity: createParticipant.isPending ? 0.6 : 1,
            }}
          >
            {createParticipant.isPending ? 'Adding...' : 'Add Participant'}
          </button>

          {createParticipant.isError && (
            <span style={{ color: '#c00', alignSelf: 'center' }}>
              Failed to add participant
            </span>
          )}
        </div>
      </div>
    </form>
  )
}

function ParticipantItem({ participant }: { participant: any }) {
  const deleteParticipant = useDeleteParticipant()
  const [showConfirm, setShowConfirm] = useState(false)

  const handleDelete = async () => {
    try {
      await deleteParticipant.mutateAsync(participant.id)
    } catch (error) {
      console.error('Failed to delete participant:', error)
    }
  }

  const statusColors: Record<string, string> = {
    registered: '#0070f3',
    confirmed: '#10b981',
    cancelled: '#ef4444',
    waitlisted: '#f59e0b',
  }

  return (
    <div style={{
      display: 'flex',
      justifyContent: 'space-between',
      alignItems: 'center',
      padding: '1rem',
      border: '1px solid #ddd',
      borderRadius: '6px',
      backgroundColor: 'white',
    }}>
      <div style={{ flex: 1 }}>
        <div style={{ fontWeight: '500', marginBottom: '0.25rem' }}>
          {participant.name}
        </div>
        <div style={{ fontSize: '0.875rem', color: '#666' }}>
          {participant.email}
        </div>
        <div style={{ 
          fontSize: '0.75rem', 
          marginTop: '0.25rem',
          display: 'inline-block',
          padding: '0.125rem 0.5rem',
          borderRadius: '4px',
          backgroundColor: statusColors[participant.status] + '20',
          color: statusColors[participant.status],
          fontWeight: '500',
        }}>
          {participant.status}
        </div>
      </div>

      <div style={{ display: 'flex', gap: '0.5rem', alignItems: 'center' }}>
        {!showConfirm ? (
          <button
            type="button"
            onClick={() => setShowConfirm(true)}
            style={{
              padding: '0.5rem 1rem',
              backgroundColor: '#fee',
              color: '#c00',
              border: '1px solid #fcc',
              borderRadius: '4px',
              fontSize: '0.875rem',
            }}
          >
            Remove
          </button>
        ) : (
          <>
            <span style={{ fontSize: '0.875rem', color: '#666' }}>Confirm?</span>
            <button
              type="button"
              onClick={handleDelete}
              disabled={deleteParticipant.isPending}
              style={{
                padding: '0.5rem 1rem',
                backgroundColor: '#c00',
                color: 'white',
                border: 'none',
                borderRadius: '4px',
                fontSize: '0.875rem',
                opacity: deleteParticipant.isPending ? 0.6 : 1,
              }}
            >
              Yes
            </button>
            <button
              type="button"
              onClick={() => setShowConfirm(false)}
              style={{
                padding: '0.5rem 1rem',
                backgroundColor: '#eee',
                color: '#333',
                border: 'none',
                borderRadius: '4px',
                fontSize: '0.875rem',
              }}
            >
              No
            </button>
          </>
        )}
      </div>
    </div>
  )
}
