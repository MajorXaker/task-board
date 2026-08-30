-- Remove the fixed slot constraint (was 1-5 only).
-- Slot is now just an ordering integer, no upper bound enforced in the DB.
ALTER TABLE boards DROP CONSTRAINT IF EXISTS boards_slot_check;
