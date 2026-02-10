import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/')({
  component: IndexComponent,
})

function IndexComponent() {
  return (
    <div style={{ padding: '2rem', fontFamily: 'system-ui, sans-serif' }}>
      <h1>Railway Test - Events</h1>
      <p>Welcome to the Railway Test frontend.</p>
      <div style={{ marginTop: '2rem' }}>
        <h2>Features</h2>
        <ul>
          <li>TanStack Start SSR</li>
          <li>TanStack Query for data fetching</li>
          <li>SSE client for real-time events</li>
          <li>API client integration</li>
        </ul>
      </div>
      <div style={{ marginTop: '2rem' }}>
        <a href="/events/1" style={{ color: '#0070f3', textDecoration: 'none' }}>
          View Event #1 →
        </a>
      </div>
    </div>
  )
}
