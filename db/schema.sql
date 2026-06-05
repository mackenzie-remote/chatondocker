-- Schema for status-service.
--
-- The deploy is responsible for loading this file into the target
-- database before the service starts. The service reads the 'region'
-- row from service_metadata when serving /api/status.

CREATE TABLE IF NOT EXISTS service_metadata (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

INSERT INTO service_metadata (key, value)
VALUES ('region', 'unknown')
ON CONFLICT (key) DO NOTHING;
