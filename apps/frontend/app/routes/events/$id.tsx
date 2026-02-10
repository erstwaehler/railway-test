import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/events/$id')({
  component: EventDetailComponent,
})

function EventDetailComponent() {
  const { id } = Route.useParams()

  return (
    <div style={{ padding: '2rem', fontFamily: 'system-ui, sans-serif' }}>
      <h1>Event Detail</h1>
      <p>Viewing event with ID: <strong>{id}</strong></p>
      <div style={{ marginTop: '2rem' }}>
        <p>Event details will be loaded from the API here.</p>
        <p>Real-time updates will be streamed via SSE.</p>
      </div>
      <div style={{ marginTop: '2rem' }}>
        <a href="/" style={{ color: '#0070f3', textDecoration: 'none' }}>
          ← Back to Home
        </a>
      </div>
    </div>
  )
}
