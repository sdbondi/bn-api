ALTER TABLE venues DROP CONSTRAINT venue_timezone_presence;
ALTER TABLE venues ALTER COLUMN timezone DROP NOT NULL;
