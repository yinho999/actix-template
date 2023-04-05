-- Add migration script here
CREATE table users
(
    id uuid NOT NULL UNIQUE,
    PRIMARY KEY (id),
    name text NOT NULL, 
    email text NOT NULL UNIQUE,
    password text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT NOW(),
    updated_at timestamptz NOT NULL DEFAULT NOW()
);