-- Create boards table
CREATE TABLE IF NOT EXISTS boards (
    id          SERIAL PRIMARY KEY,
    slot        SMALLINT NOT NULL UNIQUE CHECK (slot BETWEEN 1 AND 5),
    title       TEXT NOT NULL DEFAULT 'Board',
    color       TEXT NOT NULL DEFAULT '#4f98a3',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create boxes table
CREATE TABLE IF NOT EXISTS boxes (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    board_id    INTEGER NOT NULL REFERENCES boards(id) ON DELETE CASCADE,
    text        TEXT NOT NULL,
    color_bg    TEXT NOT NULL DEFAULT '#e8f4fd',
    color_text  TEXT NOT NULL DEFAULT '#0a3356',
    pos_x       DOUBLE PRECISION NOT NULL DEFAULT 40,
    pos_y       DOUBLE PRECISION NOT NULL DEFAULT 40,
    z_index     INTEGER NOT NULL DEFAULT 10,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Seed default 5 boards
INSERT INTO boards (slot, title, color) VALUES
    (1, 'Work',    '#4f98a3'),
    (2, 'Gaming',  '#a34f98'),
    (3, 'Board 3', '#98a34f'),
    (4, 'Board 4', '#a3734f'),
    (5, 'Board 5', '#4f5da3')
ON CONFLICT (slot) DO NOTHING;
