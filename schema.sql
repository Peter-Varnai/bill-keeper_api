-- Schema for bill_keeper API e2e testing

-- Data Groups table
CREATE TABLE IF NOT EXISTS data_groups (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    type TEXT NOT NULL CHECK (type = ANY (ARRAY['project'::text, 'organization'::text])),
    created_at VARCHAR(50) DEFAULT now(),
    bills_storage_path TEXT NOT NULL DEFAULT 'pdf_imgs/',
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
    bill INTEGER NOT NULL DEFAULT 0 REFERENCES bills(id),
    application INTEGER REFERENCES application_reports(id),
    is_cash BOOLEAN DEFAULT false
);
