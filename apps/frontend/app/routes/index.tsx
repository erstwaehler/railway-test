import { createFileRoute, Link } from '@tanstack/react-router'
import { useState } from 'react'
import { useEvents, useCreateEvent } from '../lib/queries'
import type { CreateEvent } from '../lib/types'

export const Route = createFileRoute('/')({
  component: IndexComponent,
})

function IndexComponent() {
  const { data: events, isLoading, error } = useEvents()
  const [showForm, setShowForm] = useState(false)

  return (
    <div style={{ maxWidth: '1200px', margin: '0 auto', padding: '2rem' }}>
      <header style={{ marginBottom: '2rem', borderBottom: '2px solid #eee', paddingBottom: '1rem' }}>
        <h1 style={{ fontSize: '2rem', marginBottom: '0.5rem' }}>Event Management</h1>
        <p style={{ color: '#666' }}>Manage events and participants in real-time</p>
      </header>

      <div style={{ marginBottom: '2rem' }}>
        <button
          type="button"
          onClick={() => setShowForm(!showForm)}
          style={{
            padding: '0.75rem 1.5rem',
            backgroundColor: '#0070f3',
            color: 'white',
            border: 'none',
            borderRadius: '6px',
            fontSize: '1rem',
            fontWeight: '500',
          }}
        >
          {showForm ? 'Cancel' : '+ Create New Event'}
        </button>
      </div>

      {showForm && (
        <CreateEventForm onSuccess={() => setShowForm(false)} />
      )}

      <div style={{ marginTop: '2rem' }}>
        <h2 style={{ fontSize: '1.5rem', marginBottom: '1rem' }}>Events</h2>
        
        {isLoading && <p>Loading events...</p>}
        
        {error && (
          <div style={{
            padding: '1rem',
            backgroundColor: '#fee',
            border: '1px solid #fcc',
            borderRadius: '6px',
            color: '#c00',
          }}>
            Error loading events: {error.message}
          </div>
        )}

        {events && events.length === 0 && (
          <p style={{ color: '#666', fontStyle: 'italic' }}>
            No events yet. Create your first event to get started!
          </p>
        )}

        {events && events.length > 0 && (
          <div style={{ display: 'grid', gap: '1rem' }}>
            {events.map((event) => (
              <Link
                key={event.id}
                to="/events/$id"
                params={{ id: event.id }}
                style={{
                  display: 'block',
                  padding: '1.5rem',
                  border: '1px solid #ddd',
                  borderRadius: '8px',
                  backgroundColor: 'white',
                  textDecoration: 'none',
                  color: 'inherit',
                  transition: 'all 0.2s',
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.borderColor = '#0070f3'
                  e.currentTarget.style.boxShadow = '0 4px 12px rgba(0,0,0,0.1)'
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.borderColor = '#ddd'
                  e.currentTarget.style.boxShadow = 'none'
                }}
              >
                <h3 style={{ fontSize: '1.25rem', marginBottom: '0.5rem', color: '#0070f3' }}>
                  {event.title}
                </h3>
                {event.description && (
                  <p style={{ color: '#666', marginBottom: '0.75rem' }}>
                    {event.description}
                  </p>
                )}
                <div style={{ display: 'flex', gap: '1rem', flexWrap: 'wrap', fontSize: '0.875rem', color: '#888' }}>
                  <span>📅 {new Date(event.start_time).toLocaleDateString()}</span>
                  {event.location && <span>📍 {event.location}</span>}
                  {event.max_participants && <span>👥 Max: {event.max_participants}</span>}
                </div>
              </Link>
            ))}
          </div>
        )}
      </div>
    </div>
  )
}

function CreateEventForm({ onSuccess }: { onSuccess: () => void }) {
  const createEvent = useCreateEvent()
  const [formData, setFormData] = useState<CreateEvent>({
    title: '',
    description: '',
    start_time: '',
    end_time: '',
    location: '',
    max_participants: undefined,
  })

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    
    try {
      await createEvent.mutateAsync({
        title: formData.title,
        description: formData.description || null,
        start_time: new Date(formData.start_time).toISOString(),
        end_time: new Date(formData.end_time).toISOString(),
        location: formData.location || null,
        max_participants: formData.max_participants || null,
      })
      onSuccess()
    } catch (error) {
      console.error('Failed to create event:', error)
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
        marginBottom: '2rem',
      }}
    >
      <h3 style={{ marginBottom: '1rem', fontSize: '1.25rem' }}>Create New Event</h3>
      
      <div style={{ display: 'grid', gap: '1rem' }}>
        <div>
          <label style={{ display: 'block', marginBottom: '0.25rem', fontWeight: '500' }}>
            Title *
          </label>
          <input
            type="text"
            required
            value={formData.title}
            onChange={(e) => setFormData({ ...formData, title: e.target.value })}
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
          <label style={{ display: 'block', marginBottom: '0.25rem', fontWeight: '500' }}>
            Description
          </label>
          <textarea
            value={formData.description}
            onChange={(e) => setFormData({ ...formData, description: e.target.value })}
            rows={3}
            style={{
              width: '100%',
              padding: '0.5rem',
              border: '1px solid #ddd',
              borderRadius: '4px',
              fontSize: '1rem',
              resize: 'vertical',
            }}
          />
        </div>

        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1rem' }}>
          <div>
            <label style={{ display: 'block', marginBottom: '0.25rem', fontWeight: '500' }}>
              Start Time *
            </label>
            <input
              type="datetime-local"
              required
              value={formData.start_time}
              onChange={(e) => setFormData({ ...formData, start_time: e.target.value })}
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
            <label style={{ display: 'block', marginBottom: '0.25rem', fontWeight: '500' }}>
              End Time *
            </label>
            <input
              type="datetime-local"
              required
              value={formData.end_time}
              onChange={(e) => setFormData({ ...formData, end_time: e.target.value })}
              style={{
                width: '100%',
                padding: '0.5rem',
                border: '1px solid #ddd',
                borderRadius: '4px',
                fontSize: '1rem',
              }}
            />
          </div>
        </div>

        <div style={{ display: 'grid', gridTemplateColumns: '2fr 1fr', gap: '1rem' }}>
          <div>
            <label style={{ display: 'block', marginBottom: '0.25rem', fontWeight: '500' }}>
              Location
            </label>
            <input
              type="text"
              value={formData.location}
              onChange={(e) => setFormData({ ...formData, location: e.target.value })}
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
            <label style={{ display: 'block', marginBottom: '0.25rem', fontWeight: '500' }}>
              Max Participants
            </label>
            <input
              type="number"
              min="1"
              value={formData.max_participants || ''}
              onChange={(e) => setFormData({ 
                ...formData, 
                max_participants: e.target.value ? parseInt(e.target.value) : undefined 
              })}
              style={{
                width: '100%',
                padding: '0.5rem',
                border: '1px solid #ddd',
                borderRadius: '4px',
                fontSize: '1rem',
              }}
            />
          </div>
        </div>

        <div style={{ display: 'flex', gap: '1rem', marginTop: '0.5rem' }}>
          <button
            type="submit"
            disabled={createEvent.isPending}
            style={{
              padding: '0.75rem 1.5rem',
              backgroundColor: '#0070f3',
              color: 'white',
              border: 'none',
              borderRadius: '6px',
              fontSize: '1rem',
              fontWeight: '500',
              opacity: createEvent.isPending ? 0.6 : 1,
            }}
          >
            {createEvent.isPending ? 'Creating...' : 'Create Event'}
          </button>

          {createEvent.isError && (
            <span style={{ color: '#c00', alignSelf: 'center' }}>
              Failed to create event
            </span>
          )}
        </div>
      </div>
    </form>
  )
}
