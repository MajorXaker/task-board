ALTER TABLE boards ADD CONSTRAINT boards_slot_check CHECK (slot BETWEEN 1 AND 5);
