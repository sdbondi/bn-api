ALTER TABLE events
  ADD is_external BOOLEAN NOT NULL DEFAULT FALSE,
  ADD external_url TEXT NULL;
