UPDATE venues SET timezone = 'America/Los_Angeles' WHERE timezone IS NULL;

ALTER TABLE venues ALTER COLUMN timezone SET NOT NULL;
ALTER TABLE venues ADD CONSTRAINT venue_timezone_presence CHECK (length(timezone) >= 1);
