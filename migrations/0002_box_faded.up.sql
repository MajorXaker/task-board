-- Boxes can be "faded" (semi-transparent with a dashed outline) by holding
-- the mouse/finger on them. Persisted so every client sees the same state.
ALTER TABLE boxes ADD COLUMN IF NOT EXISTS faded BOOLEAN NOT NULL DEFAULT FALSE;
