CREATE TABLE IF NOT EXISTS workers (
    id BIGSERIAL PRIMARY KEY,
    server_name TEXT NOT NULL,
    worker_name TEXT NOT NULL,
    port_name TEXT NOT NULL,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    info JSONB NULL,
    inserted_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- CREATE INDEX sever_name_idx ON workers (server_name);
-- CREATE INDEX worker_name_idx ON workers (worker_name);
-- insert into workers(server_name, worker_name, port_name, active) values('ultra', '127.0.0.1', '3000', true);
-- insert into workers(server_name, worker_name, port_name, active) values('ultra', '127.0.0.1', '3001', true);
-- insert into workers(server_name, worker_name, port_name, active) values('ultra', '127.0.0.1', '3002', true);