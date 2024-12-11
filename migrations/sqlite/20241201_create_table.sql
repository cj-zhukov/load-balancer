CREATE TABLE IF NOT EXISTS workers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    server_name TEXT NOT NULL,
    worker_name TEXT NOT NULL,
    port_name TEXT NOT NULL,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    info JSONB NULL,
    inserted_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO workers(server_name, worker_name, port_name, active) VALUES
    ('ultra', '127.0.0.1', '3001', true),
    ('ultra', '127.0.0.1', '3002', true),
    ('ultra', '127.0.0.1', '3003', true);