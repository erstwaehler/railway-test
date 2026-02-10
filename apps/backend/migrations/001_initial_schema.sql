-- Create participant_status enum type
CREATE TYPE participant_status AS ENUM ('registered', 'confirmed', 'cancelled', 'waitlisted');

-- Create events table
CREATE TABLE events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(255) NOT NULL,
    description TEXT,
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ NOT NULL,
    location VARCHAR(255),
    max_participants INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT valid_time_range CHECK (end_time > start_time),
    CONSTRAINT valid_max_participants CHECK (max_participants IS NULL OR max_participants > 0)
);

-- Create participants table
CREATE TABLE participants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL,
    status participant_status NOT NULL DEFAULT 'registered',
    registered_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT unique_event_participant UNIQUE (event_id, email)
);

-- Create indexes for better query performance
CREATE INDEX idx_events_start_time ON events(start_time);
CREATE INDEX idx_events_created_at ON events(created_at);
CREATE INDEX idx_participants_event_id ON participants(event_id);
CREATE INDEX idx_participants_email ON participants(email);
CREATE INDEX idx_participants_status ON participants(status);

-- Create function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create triggers for updated_at
CREATE TRIGGER update_events_updated_at
    BEFORE UPDATE ON events
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_participants_updated_at
    BEFORE UPDATE ON participants
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Create function to notify on event changes
CREATE OR REPLACE FUNCTION notify_event_changes()
RETURNS TRIGGER AS $$
DECLARE
    payload JSON;
BEGIN
    IF TG_OP = 'DELETE' THEN
        payload = json_build_object(
            'operation', TG_OP,
            'table', TG_TABLE_NAME,
            'id', OLD.id,
            'timestamp', NOW()
        );
        PERFORM pg_notify('event_changes', payload::text);
        RETURN OLD;
    ELSE
        payload = json_build_object(
            'operation', TG_OP,
            'table', TG_TABLE_NAME,
            'id', NEW.id,
            'data', row_to_json(NEW),
            'timestamp', NOW()
        );
        PERFORM pg_notify('event_changes', payload::text);
        RETURN NEW;
    END IF;
END;
$$ LANGUAGE plpgsql;

-- Create function to notify on participant changes
CREATE OR REPLACE FUNCTION notify_participant_changes()
RETURNS TRIGGER AS $$
DECLARE
    payload JSON;
BEGIN
    IF TG_OP = 'DELETE' THEN
        payload = json_build_object(
            'operation', TG_OP,
            'table', TG_TABLE_NAME,
            'id', OLD.id,
            'event_id', OLD.event_id,
            'timestamp', NOW()
        );
        PERFORM pg_notify('participant_changes', payload::text);
        RETURN OLD;
    ELSE
        payload = json_build_object(
            'operation', TG_OP,
            'table', TG_TABLE_NAME,
            'id', NEW.id,
            'event_id', NEW.event_id,
            'data', row_to_json(NEW),
            'timestamp', NOW()
        );
        PERFORM pg_notify('participant_changes', payload::text);
        RETURN NEW;
    END IF;
END;
$$ LANGUAGE plpgsql;

-- Create triggers for LISTEN/NOTIFY
CREATE TRIGGER event_changes_trigger
    AFTER INSERT OR UPDATE OR DELETE ON events
    FOR EACH ROW
    EXECUTE FUNCTION notify_event_changes();

CREATE TRIGGER participant_changes_trigger
    AFTER INSERT OR UPDATE OR DELETE ON participants
    FOR EACH ROW
    EXECUTE FUNCTION notify_participant_changes();
