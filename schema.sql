-- Schema for bill_keeper

-- Users table
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at VARCHAR(50) DEFAULT now()
);

-- Data Groups table
CREATE TABLE IF NOT EXISTS data_groups (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    type TEXT NOT NULL CHECK (type = ANY (ARRAY['project'::text, 'organization'::text])),
    created_at VARCHAR(50) DEFAULT now(),
    bills_storage_path TEXT NOT NULL DEFAULT 'pdf_imgs/',
    user_id INTEGER NOT NULL DEFAULT 1 REFERENCES users(id),
    UNIQUE (name)
);

-- Bills table
CREATE TABLE IF NOT EXISTS bills (
    id SERIAL PRIMARY KEY,
    data_group INTEGER NOT NULL REFERENCES data_groups(id),
    filename TEXT NOT NULL,
    amount NUMERIC(10,2),
    date DATE,
    is_cash BOOLEAN DEFAULT false
);

-- Application Reports table
CREATE TABLE IF NOT EXISTS application_reports (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    amount NUMERIC(10,2) DEFAULT 2,
    created_at VARCHAR(50) DEFAULT now(),
    submission_deadline DATE,
    data_group INTEGER NOT NULL REFERENCES data_groups(id)
);

-- Expenses table
CREATE TABLE IF NOT EXISTS expenses (
    id SERIAL PRIMARY KEY,
    data_group INTEGER NOT NULL REFERENCES data_groups(id),
    date DATE,
    partner TEXT NOT NULL,
    amount NUMERIC(10,2) NOT NULL,
    expense_type INTEGER NOT NULL DEFAULT 0,
    bill INTEGER DEFAULT 0 REFERENCES bills(id),
    application INTEGER REFERENCES application_reports(id),
    is_cash BOOLEAN DEFAULT false
);

-- Utility Data table (bank_stand, cash_stand)
CREATE TABLE IF NOT EXISTS utility_data (
    id SERIAL PRIMARY KEY,
    data_group INTEGER NOT NULL REFERENCES data_groups(id),
    key TEXT NOT NULL,
    value NUMERIC(12,2),
    created_at VARCHAR(50) DEFAULT now(),
    updated_at VARCHAR(50) DEFAULT now(),
    UNIQUE(data_group, key)
);

